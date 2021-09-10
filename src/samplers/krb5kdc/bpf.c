

// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#include <uapi/linux/ptrace.h>

struct key_t {
  char c[80];
};

// Section for function count probe: finish_process_as_req

BPF_HASH(counts_finish_process_as_req, struct key_t);

int count_finish_process_as_req(struct pt_regs *ctx) {

  if (!PT_REGS_PARM1(ctx))
    return 0;

  u64 match_val = PT_REGS_PARM1(ctx);

  if (match_val == 0) {
    struct key_t key = {.c = "NONE"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 1) {
    struct key_t key = {.c = "NAME_EXP"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 2) {
    struct key_t key = {.c = "SERVICE_EXP"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 3) {
    struct key_t key = {.c = "BAD_PVNO"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 4) {
    struct key_t key = {.c = "C_OLD_MAST_KVNO"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 5) {
    struct key_t key = {.c = "S_OLD_MAST_KVNO"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 6) {
    struct key_t key = {.c = "C_PRINCIPAL_UNKNOWN"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 7) {
    struct key_t key = {.c = "S_PRINCIPAL_UNKNOWN"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 8) {
    struct key_t key = {.c = "PRINCIPAL_NOT_UNIQUE"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 9) {
    struct key_t key = {.c = "NULL_KEY"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 10) {
    struct key_t key = {.c = "CANNOT_POSTDATE"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 11) {
    struct key_t key = {.c = "NEVER_VALID"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 12) {
    struct key_t key = {.c = "POLICY"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 13) {
    struct key_t key = {.c = "BADOPTION"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 14) {
    struct key_t key = {.c = "ETYPE_NOSUPP"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 15) {
    struct key_t key = {.c = "SUMTYPE_NOSUPP"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 16) {
    struct key_t key = {.c = "PADATA_TYPE_NOSUPP"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 17) {
    struct key_t key = {.c = "TRTYPE_NOSUPP"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 18) {
    struct key_t key = {.c = "CLIENT_REVOKED"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 19) {
    struct key_t key = {.c = "SERVICE_REVOKED"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 20) {
    struct key_t key = {.c = "TGT_REVOKED"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 21) {
    struct key_t key = {.c = "CLIENT_NOTYET"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 22) {
    struct key_t key = {.c = "SERVICE_NOTYET"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 23) {
    struct key_t key = {.c = "KEY_EXP"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 24) {
    struct key_t key = {.c = "PREAUTH_FAILED"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 25) {
    struct key_t key = {.c = "PREAUTH_REQUIRED"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 26) {
    struct key_t key = {.c = "SERVER_NOMATCH"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 27) {
    struct key_t key = {.c = "MUST_USE_USER2USER"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 28) {
    struct key_t key = {.c = "PATH_NOT_ACCEPTED"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 29) {
    struct key_t key = {.c = "SVC_UNAVAILABLE"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else {
    struct key_t key = {.c = "UNKNOWN"};
    u64 zero = 0, *count;
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
    (*count)++;
  }

  return 0;
}

// Section for function count probe: finish_dispatch_cache

BPF_HASH(counts_finish_dispatch_cache, struct key_t);

int count_finish_dispatch_cache(struct pt_regs *ctx) {

  if (!PT_REGS_PARM1(ctx))
    return 0;

  u64 match_val = PT_REGS_PARM1(ctx);

  if (match_val == 0) {
    struct key_t key = {.c = "NONE"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 1) {
    struct key_t key = {.c = "NAME_EXP"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 2) {
    struct key_t key = {.c = "SERVICE_EXP"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 3) {
    struct key_t key = {.c = "BAD_PVNO"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 4) {
    struct key_t key = {.c = "C_OLD_MAST_KVNO"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 5) {
    struct key_t key = {.c = "S_OLD_MAST_KVNO"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 6) {
    struct key_t key = {.c = "C_PRINCIPAL_UNKNOWN"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 7) {
    struct key_t key = {.c = "S_PRINCIPAL_UNKNOWN"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 8) {
    struct key_t key = {.c = "PRINCIPAL_NOT_UNIQUE"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 9) {
    struct key_t key = {.c = "NULL_KEY"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 10) {
    struct key_t key = {.c = "CANNOT_POSTDATE"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 11) {
    struct key_t key = {.c = "NEVER_VALID"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 12) {
    struct key_t key = {.c = "POLICY"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 13) {
    struct key_t key = {.c = "BADOPTION"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 14) {
    struct key_t key = {.c = "ETYPE_NOSUPP"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 15) {
    struct key_t key = {.c = "SUMTYPE_NOSUPP"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 16) {
    struct key_t key = {.c = "PADATA_TYPE_NOSUPP"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 17) {
    struct key_t key = {.c = "TRTYPE_NOSUPP"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 18) {
    struct key_t key = {.c = "CLIENT_REVOKED"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 19) {
    struct key_t key = {.c = "SERVICE_REVOKED"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 20) {
    struct key_t key = {.c = "TGT_REVOKED"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 21) {
    struct key_t key = {.c = "CLIENT_NOTYET"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 22) {
    struct key_t key = {.c = "SERVICE_NOTYET"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 23) {
    struct key_t key = {.c = "KEY_EXP"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 24) {
    struct key_t key = {.c = "PREAUTH_FAILED"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 25) {
    struct key_t key = {.c = "PREAUTH_REQUIRED"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 26) {
    struct key_t key = {.c = "SERVER_NOMATCH"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 27) {
    struct key_t key = {.c = "MUST_USE_USER2USER"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 28) {
    struct key_t key = {.c = "PATH_NOT_ACCEPTED"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 29) {
    struct key_t key = {.c = "SVC_UNAVAILABLE"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  } else {
    struct key_t key = {.c = "UNKNOWN"};
    u64 zero = 0, *count;
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
    (*count)++;
  }

  return 0;
}

// Section for function count probe: process_tgs_req

BPF_HASH(counts_process_tgs_req, struct key_t);

int count_process_tgs_req(struct pt_regs *ctx) {

  if (!PT_REGS_RC(ctx))
    return 0;

  u64 match_val = PT_REGS_RC(ctx);

  if (match_val == 0) {
    struct key_t key = {.c = "NONE"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 1) {
    struct key_t key = {.c = "NAME_EXP"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 2) {
    struct key_t key = {.c = "SERVICE_EXP"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 3) {
    struct key_t key = {.c = "BAD_PVNO"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 4) {
    struct key_t key = {.c = "C_OLD_MAST_KVNO"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 5) {
    struct key_t key = {.c = "S_OLD_MAST_KVNO"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 6) {
    struct key_t key = {.c = "C_PRINCIPAL_UNKNOWN"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 7) {
    struct key_t key = {.c = "S_PRINCIPAL_UNKNOWN"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 8) {
    struct key_t key = {.c = "PRINCIPAL_NOT_UNIQUE"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 9) {
    struct key_t key = {.c = "NULL_KEY"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 10) {
    struct key_t key = {.c = "CANNOT_POSTDATE"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 11) {
    struct key_t key = {.c = "NEVER_VALID"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 12) {
    struct key_t key = {.c = "POLICY"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 13) {
    struct key_t key = {.c = "BADOPTION"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 14) {
    struct key_t key = {.c = "ETYPE_NOSUPP"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 15) {
    struct key_t key = {.c = "SUMTYPE_NOSUPP"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 16) {
    struct key_t key = {.c = "PADATA_TYPE_NOSUPP"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 17) {
    struct key_t key = {.c = "TRTYPE_NOSUPP"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 18) {
    struct key_t key = {.c = "CLIENT_REVOKED"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 19) {
    struct key_t key = {.c = "SERVICE_REVOKED"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 20) {
    struct key_t key = {.c = "TGT_REVOKED"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 21) {
    struct key_t key = {.c = "CLIENT_NOTYET"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 22) {
    struct key_t key = {.c = "SERVICE_NOTYET"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 23) {
    struct key_t key = {.c = "KEY_EXP"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 24) {
    struct key_t key = {.c = "PREAUTH_FAILED"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 25) {
    struct key_t key = {.c = "PREAUTH_REQUIRED"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 26) {
    struct key_t key = {.c = "SERVER_NOMATCH"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 27) {
    struct key_t key = {.c = "MUST_USE_USER2USER"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 28) {
    struct key_t key = {.c = "PATH_NOT_ACCEPTED"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else if (match_val == 29) {
    struct key_t key = {.c = "SVC_UNAVAILABLE"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  } else {
    struct key_t key = {.c = "UNKNOWN"};
    u64 zero = 0, *count;
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
    (*count)++;
  }

  return 0;
}
