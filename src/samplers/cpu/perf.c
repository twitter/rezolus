// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#include <linux/ptrace.h>
#include <uapi/linux/bpf_perf_event.h>

// Arrays which hold the perf counters
BPF_PERF_ARRAY(branch_instructions_array, NUM_CPU);
BPF_PERF_ARRAY(branch_misses_array, NUM_CPU);
BPF_PERF_ARRAY(cache_misses_array, NUM_CPU);
BPF_PERF_ARRAY(cache_references_array, NUM_CPU);
BPF_PERF_ARRAY(cycles_array, NUM_CPU);
BPF_PERF_ARRAY(dtlb_load_miss_array, NUM_CPU);
BPF_PERF_ARRAY(dtlb_load_access_array, NUM_CPU);
BPF_PERF_ARRAY(dtlb_store_miss_array, NUM_CPU);
BPF_PERF_ARRAY(dtlb_store_access_array, NUM_CPU);
BPF_PERF_ARRAY(instructions_array, NUM_CPU);
BPF_PERF_ARRAY(reference_cycles_array, NUM_CPU);

// Tables which are read in user space
BPF_ARRAY(branch_instructions, u64, NUM_CPU);
BPF_ARRAY(branch_misses, u64, NUM_CPU);
BPF_ARRAY(cache_misses, u64, NUM_CPU);
BPF_ARRAY(cache_references, u64, NUM_CPU);
BPF_ARRAY(cycles, u64, NUM_CPU);
BPF_ARRAY(dtlb_load_access, u64, NUM_CPU);
BPF_ARRAY(dtlb_load_miss, u64, NUM_CPU);
BPF_ARRAY(dtlb_store_access, u64, NUM_CPU);
BPF_ARRAY(dtlb_store_miss, u64, NUM_CPU);
BPF_ARRAY(instructions, u64, NUM_CPU);
BPF_ARRAY(reference_cycles, u64, NUM_CPU);

int do_count(struct bpf_perf_event_data *ctx) {
    u32 cpu = bpf_get_smp_processor_id();
    u64 count = 0;

    count = branch_instructions_array.perf_read(CUR_CPU_IDENTIFIER);
    if ((s64)count < -256 || (s64)count > 0) {
        branch_instructions.update(&cpu, &count);
    }

    count = branch_misses_array.perf_read(CUR_CPU_IDENTIFIER);
    if ((s64)count < -256 || (s64)count > 0) {
        branch_misses.update(&cpu, &count);
    }

    count = cache_misses_array.perf_read(CUR_CPU_IDENTIFIER);
    if ((s64)count < -256 || (s64)count > 0) {
        cache_misses.update(&cpu, &count);
    }

    count = cache_references_array.perf_read(CUR_CPU_IDENTIFIER);
    if ((s64)count < -256 || (s64)count > 0) {
        cache_references.update(&cpu, &count);
    }

    count = cycles_array.perf_read(CUR_CPU_IDENTIFIER);
    if ((s64)count < -256 || (s64)count > 0) {
        cycles.update(&cpu, &count);
    }

    count = dtlb_load_access_array.perf_read(CUR_CPU_IDENTIFIER);
    if ((s64)count < -256 || (s64)count > 0) {
        dtlb_load_access.update(&cpu, &count);
    }

    count = dtlb_load_miss_array.perf_read(CUR_CPU_IDENTIFIER);
    if ((s64)count < -256 || (s64)count > 0) {
        dtlb_load_miss.update(&cpu, &count);
    }

    count = dtlb_store_access_array.perf_read(CUR_CPU_IDENTIFIER);
    if ((s64)count < -256 || (s64)count > 0) {
        dtlb_store_access.update(&cpu, &count);
    }    

    count = dtlb_store_miss_array.perf_read(CUR_CPU_IDENTIFIER);
    if ((s64)count < -256 || (s64)count > 0) {
        dtlb_store_miss.update(&cpu, &count);
    }

    count = instructions_array.perf_read(CUR_CPU_IDENTIFIER);
    if ((s64)count < -256 || (s64)count > 0) {
        instructions.update(&cpu, &count);
    }

    count = reference_cycles_array.perf_read(CUR_CPU_IDENTIFIER);
    if ((s64)count < -256 || (s64)count > 0) {
        reference_cycles.update(&cpu, &count);
    }

    return 0;
}
