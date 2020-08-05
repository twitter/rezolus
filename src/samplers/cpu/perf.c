// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

// A simple tool for tracking various cpu perf events.

#include <linux/ptrace.h>
#include <uapi/linux/bpf_perf_event.h>

// Supported events:
//
// > bpu/branch - Total branch instructions
// > bpu/miss - Total branch instructions resulting in a miss
// > cache/Access - Total cache accesses
// > cache/ciss - Total cache accesses resulting in a miss
// > cycles - CPU cycles elapsed
// > dtlb/load/access - Total dtlb loads
// > dtlb/load/miss - Total dtlb loads resulting in a miss
// > dtlb/store/access - Total dtlb stores
// > dtlb/store/miss - Total dtlb stores resulting in a miss
// > instructions - Instructions retired
// > reference_cycles - Reference number of cpu cycles elapsed
// > stalled_cycles/backend - Cylcles stalled waiting on backend
// > stalled_cycles/frontend - Cycles stalled waiting on frontend

// Change key type if you need more granular counters.
#define KEY u8
#define KEY_DEFAULT_INIT 0

// Update later with the key values you need.
static inline __attribute__((always_inline)) void get_key(KEY *key) {
    *key = 0;
}

// Add more events as needed.
BPF_HASH(bpu_branch, KEY);
int f_bpu_branch(struct bpf_perf_event_data *ctx) {
    KEY key = KEY_DEFAULT_INIT; 
	get_key(&key);
    bpu_branch.increment(key);
    return 0; 
};

BPF_HASH(bpu_miss, KEY);
int f_bpu_miss(struct bpf_perf_event_data *ctx) {
    KEY key = KEY_DEFAULT_INIT; 
	get_key(&key);
    bpu_miss.increment(key);
    return 0; 
};

BPF_HASH(cache_access, KEY);
int f_cache_access(struct bpf_perf_event_data *ctx) {
    KEY key = KEY_DEFAULT_INIT; 
	get_key(&key);
    cache_access.increment(key);
    return 0; 
};

BPF_HASH(cache_miss, KEY);
int f_cache_miss(struct bpf_perf_event_data *ctx) {
    KEY key = KEY_DEFAULT_INIT; 
	get_key(&key);
    cache_miss.increment(key);
    return 0; 
};

BPF_HASH(cycles, KEY);
int f_cycles(struct bpf_perf_event_data *ctx) {
    KEY key = KEY_DEFAULT_INIT; 
	get_key(&key);
    cycles.increment(key);
    return 0; 
};

BPF_HASH(dtlb_load_access, KEY);
int f_dtlb_load_access(struct bpf_perf_event_data *ctx) {
    KEY key = KEY_DEFAULT_INIT; 
	get_key(&key);
    dtlb_load_access.increment(key);
    return 0; 
};

BPF_HASH(dtlb_load_miss, KEY);
int f_dtlb_load_miss(struct bpf_perf_event_data *ctx) {
    KEY key = KEY_DEFAULT_INIT; 
	get_key(&key);
    dtlb_load_miss.increment(key);
    return 0; 
};

BPF_HASH(dtlb_store_access, KEY);
int f_dtlb_store_access(struct bpf_perf_event_data *ctx) {
    KEY key = KEY_DEFAULT_INIT; 
	get_key(&key);
    dtlb_store_access.increment(key);
    return 0; 
};

BPF_HASH(dtlb_store_miss, KEY);
int f_dtlb_store_miss(struct bpf_perf_event_data *ctx) {
    KEY key = KEY_DEFAULT_INIT; 
	get_key(&key);
    dtlb_store_miss.increment(key);
    return 0; 
};

BPF_HASH(instructions, KEY);
int f_instructions(struct bpf_perf_event_data *ctx) {
    KEY key = KEY_DEFAULT_INIT; 
	get_key(&key);
    instructions.increment(key);
    return 0; 
};

BPF_HASH(ref_cycles, KEY);
int f_ref_cycles(struct bpf_perf_event_data *ctx) {
    KEY key = KEY_DEFAULT_INIT; 
	get_key(&key);
    ref_cycles.increment(key);
    return 0; 
};

BPF_HASH(stalled_backend, KEY);
int f_stalled_backend(struct bpf_perf_event_data *ctx) {
    KEY key = KEY_DEFAULT_INIT; 
	get_key(&key);
    stalled_backend.increment(key);
    return 0; 
};

BPF_HASH(stalled_frontend, KEY);
int f_stalled_frontend(struct bpf_perf_event_data *ctx) {
    KEY key = KEY_DEFAULT_INIT; 
	get_key(&key);
    stalled_frontend.increment(key);
    return 0; 
};
