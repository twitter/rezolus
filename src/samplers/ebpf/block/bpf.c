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
BPF_HISTOGRAM(read_size, int, 461);
BPF_HISTOGRAM(read_latency, int, 461);
BPF_HISTOGRAM(read_request_latency, int, 461);
BPF_HISTOGRAM(read_queue_latency, int, 461);
BPF_HISTOGRAM(write_size, int, 461);
BPF_HISTOGRAM(write_latency, int, 461);
BPF_HISTOGRAM(write_request_latency, int, 461);
BPF_HISTOGRAM(write_queue_latency, int, 461);
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
            write_queue_latency.increment(index);
        } else {
            read_queue_latency.increment(index);
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
            write_size.increment(index);
        } else {
            read_size.increment(index);
        }
    }

    // Latency
    u64 *enqueued, *requested;

    // total latency including queued time
    enqueued = queue_start.lookup(&req);
    if (enqueued != 0) {
        unsigned int index = value_to_index2((now - *enqueued) / 1000);
        if (rwflag == 1) {
            write_latency.increment(index);
        } else {
            read_latency.increment(index);
        }
    }

    // request latency not including queued time
    requested = request_start.lookup(&req);
    if (requested != 0) {
        unsigned int index = value_to_index2((now - *requested) / 1000);
        if (rwflag == 1) {
            write_request_latency.increment(index);
        } else {
            read_request_latency.increment(index);
        }
    }

    return 0;
}
