#include <uapi/linux/ptrace.h>
#include <linux/blkdev.h>

struct val_t {
    char name[TASK_COMM_LEN];
};

// value_to_index2() gives us from 0-460 as the index
BPF_HISTOGRAM(dist, int, 461);
BPF_HASH(commbyreq, struct request *, struct val_t);

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
        commbyreq.update(&req, &val);
    }
    return 0;
}

int do_count(struct pt_regs *ctx, struct request *req)
{
    struct val_t *valp;
    valp = commbyreq.lookup(&req);
    if (valp == 0) {
       return 0;
    }
    u64 delta = req->__data_len / 1024;
    unsigned int index = value_to_index2(delta);
    if (req->__data_len > 0) {
        dist.increment(index);
    }
    return 0;
}
