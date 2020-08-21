// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#include <linux/ptrace.h>
#include <uapi/linux/bpf_perf_event.h>

// Arrays which hold the perf counters
BPF_PERF_ARRAY(cpu_migrations_array, NUM_CPU);


// Tables which are read in user space
BPF_ARRAY(cpu_migrations, u64, NUM_CPU);


int do_count(struct bpf_perf_event_data *ctx) {
    u32 cpu = bpf_get_smp_processor_id();
    u64 count = 0;

    count = cpu_migrations_array.perf_read(CUR_CPU_IDENTIFIER);
    if ((s64)count < -256 || (s64)count > 0) {
        cpu_migrations.update(&cpu, &count);
    }

    return 0;
}
