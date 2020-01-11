// Based on: https://github.com/iovisor/bcc/blob/master/tools/runqlat.py

#include <uapi/linux/ptrace.h>
#include <linux/sched.h>
#include <linux/nsproxy.h>
#include <linux/pid_namespace.h>

typedef struct pid_key {
    u64 id;
    u64 slot;
} pid_key_t;

typedef struct pidns_key {
    u64 id;
    u64 slot;
} pidns_key_t;

BPF_TABLE("hash", u32, u64, start, 65536);

// value_to_index() gives us from 0-460 as the index
BPF_HISTOGRAM(runqueue_latency, int, 461);

struct rq;

// from /sys/kernel/debug/tracing/events/sched/sched_wakeup/format
struct sched_wakeup_arg {
    u64 __unused__;
    char comm[16];
    pid_t pid;
    int prio;
    int success;
    int target_cpu;
};

static int trace_enqueue(u32 tgid, u32 pid)
{
    u64 ts = bpf_ktime_get_ns();
    start.update(&pid, &ts);
    return 0;
}

int trace_wake_up_new_task(struct pt_regs *ctx, struct task_struct *p)
{
    return trace_enqueue(p->tgid, p->pid);
}

int trace_ttwu_do_wakeup(struct pt_regs *ctx, struct rq *rq, struct task_struct *p,
    int wake_flags)
{
    return trace_enqueue(p->tgid, p->pid);
}

// from /sys/kernel/debug/tracing/events/sched/sched_switch/format
struct sched_switch_arg {
    u64 __unused__;
    char prev_comm[16];
    pid_t prev_pid;
    int prev_prio;
    long prev_state;
    char next_comm[16];
    pid_t next_pid;
    int next_prio;
};

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

int trace_run(struct pt_regs *ctx, struct task_struct *prev)
{
    // handle involuntary context switch
    if (prev->state == TASK_RUNNING) {
        u32 tgid = prev->tgid;
        u32 pid = prev->pid;
        u64 ts = bpf_ktime_get_ns();
        start.update(&pid, &ts);
    }

    // get tgid and pid
    u32 tgid = bpf_get_current_pid_tgid() >> 32;
    u32 pid = bpf_get_current_pid_tgid();

    // lookup start time
    u64 *tsp = start.lookup(&pid);

    // skip events with unknown start
    if (tsp == 0) {
        return 0;
    }

    // calculate latency in microseconds
    u64 delta = (bpf_ktime_get_ns() - *tsp) / 1000;

    // calculate index and increment histogram
    unsigned int index = value_to_index2(delta);
    runqueue_latency.increment(index);

    // clear the start time
    start.delete(&pid);
    return 0;
}
