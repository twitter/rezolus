// A simple tool for tracking various cpu perf events.
//
// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#include <linux/ptrace.h>
#include <uapi/linux/bpf_perf_event.h>

// Supported events:
//
// > BPU/Branch - Total branch instructions
// > BPU/Miss - Total branch instructions resulting in a miss
// > Cache/Access - Total cache accesses
// > Cache/Miss - Total cache accesses resulting in a miss
// > Cycles - cpu cyles elapsed
// > Dtlb/load/access - total dtlb loads
// > Dtlb/load/miss - total dtlb loads resulting in a miss
// > Dtlb/store/access - total dtlb stores
// > Dtlb/store/miss - total dtlb stores resulting in a miss
// > instructions - instructions retired
// > Reference_cycles - reference number of cpu cycles elapsed
// > stalled_cycles/backend - cylcles stalled waiting on backend
// > stalled_cyles/frontend - cycles stalled waiting on frontend

// Change key type if you need a more granular counters.
#define KEY u8
#define KEY_DEFAULT 0
#define COUNTER(name) BPF_HASH(name, KEY)   \
int name(struct bpf_perf_event_data *ctx) { \
    KEY key = KEY_DEFAULT;                  \
    get_key(&key);                          \
    name.increment(&key);                   \
    return 0;                               \
}

// Update later with the key values you need.
static inline __attribute__((always_inline)) void get_key(KEY *key) {
    *key = 0;
}

// Add more events as needed.
COUNTER(bpu_branch);
COUNTER(bpu_miss);
COUNTER(cache_access,;
COUNTER(cache_miss);
COUNTER(cycles);
COUNTER(dtlb_load_access);
COUNTER(dtlb_load_miss);
COUNTER(dtlb_store_access,;
COUNTER(dtlb_store_miss);
COUNTER(instructions);
COUNTER(ref_cycles);
COUNTER(stalled_backend);
COUNTER(stalled_frontend);
