#include <uapi/linux/ptrace.h>
#include <linux/irq.h>
#include <linux/irqdesc.h>
#include <linux/interrupt.h>

// This code is taken from: 
//      https://github.com/iovisor/bcc/blob/master/tools/hardirqs.py
//      https://github.com/iovisor/bcc/blob/master/tools/softirqs.py
//
// Copyright (c) 2015 Brendan Gregg.
// Licensed under the Apache License, Version 2.0 (the "License")

typedef struct account_val {
    u64 ts;
    u32 vec;
} account_val_t;

// Software IRQ
BPF_HASH(soft_start, u32, account_val_t);
BPF_HISTOGRAM(hi, int, 461);
BPF_HISTOGRAM(timer, int, 461);
BPF_HISTOGRAM(net_tx, int, 461);
BPF_HISTOGRAM(net_rx, int, 461);
BPF_HISTOGRAM(block, int, 461);
BPF_HISTOGRAM(irq_poll, int, 461);
BPF_HISTOGRAM(tasklet, int, 461);
BPF_HISTOGRAM(sched, int, 461);
BPF_HISTOGRAM(hr_timer, int, 461);
BPF_HISTOGRAM(rcu, int, 461);
BPF_HISTOGRAM(unknown, int, 461);

// Hardware IRQ
BPF_HASH(hard_start, u32, u64);
BPF_HISTOGRAM(hardirq_total, int, 461);

VALUE_TO_INDEX2_FUNC

// Software IRQ
int softirq_entry(struct tracepoint__irq__softirq_entry *args)
{
    u32 pid = bpf_get_current_pid_tgid();
    account_val_t val = {};
    val.ts = bpf_ktime_get_ns();
    val.vec = args->vec;
    soft_start.update(&pid, &val);
    return 0;
}

// For bcc 0.7.0 + 
int softirq_exit(struct tracepoint__irq__softirq_exit *args)
{
    u64 delta_us;
    u32 vec;
    u32 pid = bpf_get_current_pid_tgid();
    account_val_t *valp;

    // fetch timestamp and calculate delta
    valp = soft_start.lookup(&pid);
    if (valp == 0) {
        return 0;   // missed start
    }
    delta_us = (bpf_ktime_get_ns() - valp->ts) / 1000ul;
    vec = valp->vec;
    u64 index = value_to_index2(delta_us);

    // May need updates if more softirqs are added
    switch (vec) {
        case 0: hi.increment(index); break;
        case 1: timer.increment(index); break;
        case 2: net_tx.increment(index); break;
        case 3: net_rx.increment(index); break;
        case 4: block.increment(index); break;
        case 5: irq_poll.increment(index); break;
        case 6: tasklet.increment(index); break;
        case 7: sched.increment(index); break;
        case 8: hr_timer.increment(index); break;
        case 9: rcu.increment(index); break;
        default: unknown.increment(index); break;
    }

    soft_start.delete(&pid);
    return 0;
}

// Hardware IRQ
int hardirq_entry(struct pt_regs *ctx, struct irq_desc *desc)
{
    u32 pid = bpf_get_current_pid_tgid();
    u64 ts = bpf_ktime_get_ns();
    hard_start.update(&pid, &ts);
    return 0;
}

int hardirq_exit(struct pt_regs *ctx)
{
    u64 *tsp, delta_us, index;
    u32 pid = bpf_get_current_pid_tgid();

    // fetch timestamp and calculate delta
    tsp = hard_start.lookup(&pid);
    if (tsp == 0 ) {
        return 0;   // missed start
    }
   
    delta_us = (bpf_ktime_get_ns() - *tsp) / 1000ul;
    index = value_to_index2(delta_us);
    hardirq_total.increment(index);

    hard_start.delete(&pid);
    return 0;
}
