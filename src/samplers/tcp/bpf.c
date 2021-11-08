// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

// Based on: https://github.com/iovisor/bcc/blob/master/tools/tcpconnlat.py

#include <uapi/linux/ptrace.h>
#include <net/sock.h>
#include <net/tcp_states.h>
#include <net/inet_sock.h>
#include <bcc/proto.h>
#include <linux/tcp.h>

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
    // for 4.10+, sk_gso_max_segs is following sk_protocol field.
    // in order to be compatiable, we handle both cases.
    // we calculate the offset between sk_lingertime_offset and gso_max_segs_offset
    // and for 4.10+, the offset is 4, otherwise it's pre-4.10.
    // see details in -> https://github.com/iovisor/bcc/blob/04893e3bb1c03a97f6ea3835986abe6608062f6a/tools/tcpaccept.py#L120
    int gso_max_segs_offset = offsetof(struct sock, sk_gso_max_segs);
    int sk_lingertime_offset = offsetof(struct sock, sk_lingertime);

    // get the sk_protocol bitfield
    if (sk_lingertime_offset - gso_max_segs_offset == 4)
        // 4.10+ with little endian
#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
        // get the sk_protocol bitfield, see https://elixir.bootlin.com/linux/v5.4/source/include/net/sock.h#L455.
        protocol = *(u8 *)((u64)&sk->sk_gso_max_segs - 3); 
    else
        // pre-4.10 with little endian
        protocol = *(u8 *)((u64)&sk->sk_wmem_queued - 3);
#elif __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
        // 4.10+ with big endian
        protocol = *(u8 *)((u64)&sk->sk_gso_max_segs - 1);
    else
        // pre-4.10 with big endian
        protocol = *(u8 *)((u64)&sk->sk_wmem_queued - 1);
#else
#error "Fix your compiler's __BYTE_ORDER__?!"
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
    struct sock_stats_t *stats = sock_stats_map.lookup(&sk);
    if (!stats)
        return 0; // missed entry or filtered

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