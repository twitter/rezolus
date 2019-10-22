// Based on: https://github.com/iovisor/bcc/blob/master/tools/tcpconnlat.py

#include <uapi/linux/ptrace.h>
#include <net/sock.h>
#include <net/tcp_states.h>
#include <bcc/proto.h>

struct info_t {
    u64 ts;
    u32 pid;
    char task[TASK_COMM_LEN];
};

BPF_HASH(start, struct sock *, struct info_t);

BPF_HISTOGRAM(connlat, int, 461);

// histogram indexing
static unsigned int value_to_index2(unsigned int value) {
    unsigned int index = 460;
    if (value < 100) {
        // 0-99 => [0..100)
        // 0 => 0
        // 99 => 99
        index = value;
    } else if (value < 1000) {
        // 100-999 => [100..190)
        // 100 => 100
        // 999 => 189
        index = 90 + value / 10;
    } else if (value < 10000) {
        // 1_000-9_999 => [190..280)
        // 1000 => 190
        // 9999 => 279
        index = 180 + value / 100;
    } else if (value < 100000) {
        // 10_000-99_999 => [280..370)
        // 10000 => 280
        // 99999 => 369
        index = 270 + value / 1000;
    } else if (value < 1000000) {
        // 100_000-999_999 => [370..460)
        // 100000 => 370
        // 999999 => 459
        index = 360 + value / 10000;
    } else {
        index = 460;
    }
    return index;
}

int trace_connect(struct pt_regs *ctx, struct sock *sk)
{
    u32 pid = bpf_get_current_pid_tgid();
    struct info_t info = {.pid = pid};
    info.ts = bpf_ktime_get_ns();
    bpf_get_current_comm(&info.task, sizeof(info.task));
    start.update(&sk, &info);
    return 0;
};

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
    struct info_t *infop = start.lookup(&skp);
    if (infop == 0) {
        return 0;   // missed entry or filtered
    }
    u64 ts = infop->ts;
    u64 now = bpf_ktime_get_ns();
    u64 delta_us = (now - ts) / 1000ul;
    u64 index = value_to_index2(delta_us);
    connlat.increment(index);

    start.delete(&skp);
    return 0;
}