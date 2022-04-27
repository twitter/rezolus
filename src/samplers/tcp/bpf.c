// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

// Based on: https://github.com/iovisor/bcc/blob/master/tools/tcpconnlat.py

#include <uapi/linux/ptrace.h>
#include <net/sock.h>
#include <net/tcp_states.h>
#include <net/inet_sock.h>
#include <bcc/proto.h>
#include <linux/tcp.h>
#include <net/tcp.h>
#include <linux/version.h>

// stores the stats of a connection.
struct sock_stats_t {
    char task[TASK_COMM_LEN];           // process name
    u64 ts;                             // the starting timestamp of this connection
    u64 pid;                            // TGID/PID
} __attribute__((packed));              // minimize the memory needed.

// Map pid to the function param to map kprobe to kretprobe
BPF_HASH(param_map, u64, u64);
// Map a tcp connection to its statistics
BPF_HASH(sock_stats_map, struct sock *, struct sock_stats_t);

// histograms
BPF_HISTOGRAM(connlat, int, 461);
BPF_HISTOGRAM(srtt, int, 461);
BPF_HISTOGRAM(jitter, int, 461);

// counters
BPF_ARRAY(conn_accepted, u64, 1);
BPF_ARRAY(conn_initiated, u64, 1);
BPF_ARRAY(drop, u64, 1);
BPF_ARRAY(tlp, u64, 1);
BPF_ARRAY(rto, u64, 1);
BPF_ARRAY(duplicate, u64, 1);
BPF_ARRAY(ooo, u64, 1);

// store a pointer by the pid
static void store_ptr(u64 pid, u64 ptr)
{
    param_map.update(&pid, &ptr);
}

// fetch the pointer by the pid and remove the pid from the map.
static u64 fetch_ptr(u64 pid)
{
    u64 *ptr = param_map.lookup(&pid);
    if (!ptr) {
        return 0;
    }
    u64 val = *ptr;
    param_map.delete(&pid);
    return val;
}

// helper function to get srtt from tcp and copy to BPF space from kernel space.
static u32 get_srtt_us(struct tcp_sock *ts) {
    u32 srtt_us = 0;
    bpf_probe_read_kernel(&srtt_us, sizeof(srtt_us), (void *)&ts->srtt_us);
    // we do >> 3 because the value recorded in ts->srtt_us is actually 8 times
    // the value of real srtt for easier calculation.
    // see the thread in: https://lkml.org/lkml/1998/9/12/41
    // and source code in: https://elixir.bootlin.com/linux/latest/source/net/ipv4/tcp_input.c#L797
    return srtt_us >> 3;
}

// helper function to get median deviation of srtt from tcp and copy to BPF space from kernel space.
static u32 get_jitter_us(struct tcp_sock *ts) {
    u32 mdev_us = 0;
    bpf_probe_read_kernel(&mdev_us, sizeof(mdev_us), (void *)&ts->mdev_us);
    // we do >> 2 because the value recorded in ts->mdev_us is actually 4 times
    // the value of real mdev_us for easier calculation.
    // see source code in: https://elixir.bootlin.com/linux/latest/source/net/ipv4/tcp_input.c#L838
    return mdev_us >> 2;
}

// helper function to add value aotmically.
static void add_value(u64* val, u64 delta) {
    if (val)
        lock_xadd(val, delta);
}

VALUE_TO_INDEX2_FUNC

// kprobe handler for tcp_v4_connect and tcp_v6_connect
int trace_connect(struct pt_regs *ctx, struct sock *sk)
{
    struct sock_stats_t stats = {.pid = bpf_get_current_pid_tgid()};
    stats.ts = bpf_ktime_get_ns();
    bpf_get_current_comm(&stats.task, sizeof(stats.task));
    // store the sock's stats.
    sock_stats_map.update(&sk, &stats);
    // store the sock's pointer to the pid so it can be used in return handler later.
    store_ptr(stats.pid, (u64)sk);

    return 0;
};

// kretprobe handler for tcp_v4_connect and tcp_v6_connect's return.
int trace_connect_return(struct pt_regs *ctx)
{
    // get the sock from the param_map we saved in trace_connect
    struct sock *sk = (struct sock *)fetch_ptr(bpf_get_current_pid_tgid());
    if (!sk)
        return 0;
    int ret = PT_REGS_RC(ctx);
    // Non-zero retcode means the connection failed right away.
    if (ret != 0) {
        // clean up.
        struct sock_stats_t *stats = sock_stats_map.lookup(&sk);
        if (!stats)
            return 0;
        sock_stats_map.delete(&sk);
    }
    return 0;
}

// kprobe handler for tcp_finish_connect
int trace_finish_connect(struct pt_regs *ctx, struct sock *sk, struct sk_buff *skb)
{
    struct sock_stats_t *stats = sock_stats_map.lookup(&sk);
    if (!stats) {
        return 0;
    }
    // increment counter
    int loc = 0;
    add_value(conn_initiated.lookup(&loc), 1);

    return 0;
}

// kprobe handler for tcp_set_state
int trace_tcp_set_state(struct pt_regs *ctx, struct sock *sk, int state)
{
    // We only handle closed connection, so early exist for non close ones.
    if (state != TCP_CLOSE)
        return 0;

    // cleanup the connection since it's closed.
    struct sock_stats_t *stats = sock_stats_map.lookup(&sk);
    if (!stats) {
        return 0;
    }
    sock_stats_map.delete(&sk);
    return 0;
}

// kretprobe handler for inet_socket_accept's return.
int trace_inet_socket_accept_return(struct pt_regs *ctx)
{
    // inet_socket_accept returns sock* directly, so we get from PT_REGS_RC.
    struct sock *sk = (struct sock *)PT_REGS_RC(ctx);
    if (!sk)
        return 0;

    // check this is TCP
    u8 protocol = 0;
    // unfortunately, we need to have different handling for pre-4.10 and 4.10+
    // for pre-4.10, sk_wmem_queued is following sk_protocol field.
    // for 4.10+ to 5.5, sk_gso_max_segs is following sk_protocol field.
    // for 5.6+, sk_gso_max_segs is following sk_protocol field, but the sk_protocol becomes regular member
    // instead of bitfield.
    // in order to be compatible, we handle all cases.
#if LINUX_VERSION_CODE >= KERNEL_VERSION(5,6,0)
    // 5.6+, we can read sk_protocol as a regular field.
    u16 p = 0;
    bpf_probe_read_kernel(&p, sizeof(p), ((const char*)sk) +
        offsetof(struct sock, sk_protocol));
    protocol = (u8) p;
#elif LINUX_VERSION_CODE >= KERNEL_VERSION(4,10,0)
    // from 4.10+ to 5.5, sk_protocol is a bit field
    // see https://elixir.bootlin.com/linux/v5.4/source/include/net/sock.h#L455.
    #if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
        // get the sk_protocol bitfield
        protocol = *(u8 *)((u64)&sk->sk_gso_max_segs - 3);
    #elif __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
        protocol = *(u8 *)((u64)&sk->sk_gso_max_segs - 1);
    #else
    #error "Fix your compiler's __BYTE_ORDER__?!"
    #endif
#else
    // for pre-4.10, sk_protocol is also a bit field
    #if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
        protocol = *(u8 *)((u64)&sk->sk_wmem_queued - 3);
    #elif __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
        protocol = *(u8 *)((u64)&sk->sk_wmem_queued - 1);
    #else
    #error "Fix your compiler's __BYTE_ORDER__?!"
    #endif
#endif

    // if the sock is not TCP, igonre.
    if (protocol != IPPROTO_TCP)
        return 0;

    // create the sock stats for the new accepted connection.
    struct sock_stats_t stats = {.pid = bpf_get_current_pid_tgid()};
    struct tcp_sock *ts = tcp_sk(sk);
    // we approximate the starting time to be current time minus the srtt.
    stats.ts = bpf_ktime_get_ns() - get_srtt_us(ts) * 1000;
    bpf_get_current_comm(&stats.task, sizeof(stats.task));
    // store the sock's stats.
    sock_stats_map.update(&sk, &stats);

    // increment counter;
    int loc = 0;
    add_value(conn_accepted.lookup(&loc), 1);

    return 0;
}

// See tcp_v4_do_rcv() and tcp_v6_do_rcv(). So TCP_ESTBALISHED and TCP_LISTEN
// are fast path and processed elsewhere, and leftovers are processed by
// tcp_rcv_state_process(). We can trace this for handshake completion.
// This should all be switched to static tracepoints when available.
int trace_tcp_rcv_state_process(struct pt_regs *ctx, struct sock *skp)
{
    // will be in TCP_SYN_SENT for handshake
    if (skp->__sk_common.skc_state != TCP_SYN_SENT)
        return 0;
    // check start and calculate delta
    struct sock_stats_t *stats = sock_stats_map.lookup(&skp);
    if (stats == 0) {
        return 0;   // missed entry or filtered
    }
    u64 ts = stats->ts;
    u64 now = bpf_ktime_get_ns();
    u64 delta_us = (now - ts) / 1000ul;
    u64 index = value_to_index2(delta_us);
    connlat.increment(index);

    return 0;
}

// this is actually the fast path, we need to watch out for the overhead added here.
int trace_tcp_rcv(struct pt_regs *ctx, struct sock *sk, struct sk_buff *skb)
{
    if (sk == NULL)
        return 0;

    struct tcp_sock *ts = tcp_sk(sk);
    // update srtt and jitter.
    srtt.increment(value_to_index2(get_srtt_us(ts)));
    jitter.increment(value_to_index2(get_jitter_us(ts)));

    return 0;
}

int trace_tcp_drop(struct pt_regs *ctx, struct sock *sk, struct sk_buff *skb)
{
    if (sk == NULL)
        return 0;
    int loc = 0;
    add_value(drop.lookup(&loc), 1);
    return 0;
}

// Count the amount of Tail Loss Recovery Probes (TLP)
int trace_tlp(struct pt_regs *ctx, struct sock *sk) 
{
    if (sk == NULL)
        return 0;
    int loc = 0;
    add_value(tlp.lookup(&loc), 1);
    return 0;
}

// Count the amount of Retransmission Timeouts (RTO)
int trace_rto(struct pt_regs *ctx, struct sock *sk) 
{
    if (sk == NULL)
        return 0;
    int loc = 0;
    add_value(rto.lookup(&loc), 1);
    return 0;
}

// Run on incoming package validation
int trace_validate_incoming(struct pt_regs *ctx, struct sock *sk, struct sk_buff *skb) {
    if (sk == NULL || skb == NULL)
        return 0;

    // read seq and rcv_nxt from kernel to bpf.
    u32 seq = 0;
    bpf_probe_read_kernel(&seq, sizeof(seq), ((const char *)skb) +
               offsetof(struct sk_buff, cb) +
               offsetof(struct tcp_skb_cb, seq));
    u32 rcv_nxt = 0;
    bpf_probe_read_kernel(&rcv_nxt, sizeof(rcv_nxt), ((const char *)sk) +
               offsetof(struct tcp_sock, rcv_nxt));

    int64_t distance = (int64_t)(seq) - (int64_t)(rcv_nxt);

    // Segment sequence before the expected one
    // which means this was a duplicated segment
    if (distance < 0) {
        // Increment duplicate counter
        int loc = 0;
        add_value(duplicate.lookup(&loc), 1);
    }

    // Segment sequence after the expected one
    // which means this segment was received out of order
    if (distance > 0) {
        // Increment out of order counter
        int loc = 0;
        add_value(ooo.lookup(&loc), 1);
    }
    return 0;
}
