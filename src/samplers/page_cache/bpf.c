#include <uapi/linux/ptrace.h>

BPF_ARRAY(page_accessed, u64, 1);
BPF_ARRAY(buffer_dirty, u64, 1);
BPF_ARRAY(add_to_page_cache_lru, u64, 1);
BPF_ARRAY(page_dirtied, u64, 1);

int trace_mark_page_accessed(struct pt_regs *ctx)
{
    u64 *count = page_accessed.lookup(&op);
    if (count) lock_xadd(count, 1);
    return 0;
}

int trace_mark_buffer_dirty(struct pt_regs *ctx)
{
    u64 *count = buffer_dirty.lookup(&op);
    if (count) lock_xadd(count, 1);
    return 0;
}

int trace_add_to_page_cache_lru(struct pt_regs *ctx)
{
    u64 *count = add_to_page_cache_lru.lookup(&op);
    if (count) lock_xadd(count, 1);
    return 0;
}

int trace_account_page_dirtied(struct pt_regs *ctx)
{
    u64 *count = page_dirtied.lookup(&op);
    if (count) lock_xadd(count, 1);
    return 0;
}
