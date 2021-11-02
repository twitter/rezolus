// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#include <uapi/linux/ptrace.h>
#include <linux/blkdev.h>

struct val_t {
    char name[TASK_COMM_LEN];
};

// hashes to track request details
BPF_HASH(queue_start, struct request *);
BPF_HASH(request_start, struct request *);
BPF_HASH(commbyreq, struct request *, struct val_t);

// value_to_index2() gives us from 0-460 as the index
BPF_HISTOGRAM(io_size_read, int, 461);
BPF_HISTOGRAM(latency_read, int, 461);
BPF_HISTOGRAM(device_latency_read, int, 461);
BPF_HISTOGRAM(queue_latency_read, int, 461);
BPF_HISTOGRAM(io_size_write, int, 461);
BPF_HISTOGRAM(latency_write, int, 461);
BPF_HISTOGRAM(device_latency_write, int, 461);
BPF_HISTOGRAM(queue_latency_write, int, 461);

VALUE_TO_INDEX2_FUNC

int trace_pid_start(struct pt_regs *ctx, struct request *req)
{
    struct val_t val = {};
    if (bpf_get_current_comm(&val.name, sizeof(val.name)) == 0) {
        u64 ts = bpf_ktime_get_ns();
        queue_start.update(&req, &ts);
        commbyreq.update(&req, &val);
    }
    return 0;
}

int trace_req_start(struct pt_regs *ctx, struct request *req)
{
    u64 now = bpf_ktime_get_ns();

    u64 rwflag = 0;
    #ifdef REQ_WRITE
        rwflag = !!(req->cmd_flags & REQ_WRITE);
    #elif defined(REQ_OP_SHIFT)
        rwflag = !!((req->cmd_flags >> REQ_OP_SHIFT) == REQ_OP_WRITE);
    #else
        rwflag = !!((req->cmd_flags & REQ_OP_MASK) == REQ_OP_WRITE);
    #endif

    u64 *enqueued;
    enqueued = queue_start.lookup(&req);
    if (enqueued != 0) {
        unsigned int index = value_to_index2((now - *enqueued) / 1000);
        if (rwflag == 1) {
            queue_latency_write.increment(index);
        } else {
            queue_latency_read.increment(index);
        }
    }
    request_start.update(&req, &now);
    return 0;
}

int do_count(struct pt_regs *ctx, struct request *req)
{
    u64 now = bpf_ktime_get_ns();

    u64 rwflag = 0;
    #ifdef REQ_WRITE
        rwflag = !!(req->cmd_flags & REQ_WRITE);
    #elif defined(REQ_OP_SHIFT)
        rwflag = !!((req->cmd_flags >> REQ_OP_SHIFT) == REQ_OP_WRITE);
    #else
        rwflag = !!((req->cmd_flags & REQ_OP_MASK) == REQ_OP_WRITE);
    #endif

    // Size
    struct val_t *valp;
    valp = commbyreq.lookup(&req);
    if (valp == 0) {
       return 0;
    }
    u64 delta = req->__data_len / 1024;
    unsigned int index = value_to_index2(delta);
    if (req->__data_len > 0) {
        if (rwflag == 1) {
            io_size_write.increment(index);
        } else {
            io_size_read.increment(index);
        }
    }

    // Latency
    u64 *enqueued, *requested;

    // total latency including queued time
    enqueued = queue_start.lookup(&req);
    if (enqueued != 0) {
        unsigned int index = value_to_index2((now - *enqueued) / 1000);
        if (rwflag == 1) {
            latency_write.increment(index);
        } else {
            latency_read.increment(index);
        }
    }

    // request latency not including queued time
    requested = request_start.lookup(&req);
    if (requested != 0) {
        unsigned int index = value_to_index2((now - *requested) / 1000);
        if (rwflag == 1) {
            device_latency_write.increment(index);
        } else {
            device_latency_read.increment(index);
        }
    }

    return 0;
}
