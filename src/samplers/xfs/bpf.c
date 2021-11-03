// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

// Based on: https://github.com/iovisor/bcc/blob/master/tools/xfsdist.py

#include <uapi/linux/ptrace.h>
#include <linux/fs.h>
#include <linux/sched.h>

#define OP_NAME_LEN 8

typedef struct dist_key {
    char op[OP_NAME_LEN];
    u64 slot;
} dist_key_t;

BPF_HASH(start, u32);

// value_to_index2() gives us from 0-460 as the index
BPF_HISTOGRAM(read, int, 461);
BPF_HISTOGRAM(write, int, 461);
BPF_HISTOGRAM(open, int, 461);
BPF_HISTOGRAM(fsync, int, 461);

VALUE_TO_INDEX2_FUNC

int trace_entry(struct pt_regs *ctx)
{
    u32 pid = bpf_get_current_pid_tgid();
    u64 ts = bpf_ktime_get_ns();
    start.update(&pid, &ts);
    return 0;
}

static int trace_return(struct pt_regs *ctx, int op)
{
    // get pid
    u32 pid = bpf_get_current_pid_tgid();

    // lookup start time
    u64 *tsp = start.lookup(&pid);

    // skip events without start
    if (tsp == 0) {
        return 0;
    }

    // calculate latency in microseconds
    u64 delta = (bpf_ktime_get_ns() - *tsp) / 1000;

    // calculate index
    u64 index = value_to_index2(delta);

    // store into correct histogram for OP
    if (op == 0) {
        read.increment(index);
    } else if (op == 1) {
        write.increment(index);
    } else if (op == 2) {
        open.increment(index);
    } else if (op == 3) {
        fsync.increment(index);
    }

    // clear the start time
    start.delete(&pid);

    return 0;
}

int trace_read_return(struct pt_regs *ctx)
{
    return trace_return(ctx, 0);
}

int trace_write_return(struct pt_regs *ctx)
{
    return trace_return(ctx, 1);
}

int trace_open_return(struct pt_regs *ctx)
{
    return trace_return(ctx, 2);
}

int trace_fsync_return(struct pt_regs *ctx)
{
    return trace_return(ctx, 3);
}
