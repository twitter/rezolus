// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#include <uapi/linux/ptrace.h>

BPF_HISTOGRAM(rx_size, int, 461);
BPF_HISTOGRAM(tx_size, int, 461);

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

int trace_transmit(struct tracepoint__net__net_dev_queue *args)
{
    u64 index = value_to_index2(args->len);
    tx_size.increment(index);
    return 0;
}

int trace_receive(struct tracepoint__net__netif_rx *args)
{
    u64 index = value_to_index2(args->len);
    rx_size.increment(index);
    return 0;
}
