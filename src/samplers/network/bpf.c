// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#include <uapi/linux/ptrace.h>

BPF_HISTOGRAM(rx_size, int, 461);
BPF_HISTOGRAM(tx_size, int, 461);

VALUE_TO_INDEX2_FUNC

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
