

// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

// krb_error_codes are an old format and translated into a platform appropriate
// format to be passed around on the stack. This is why the krb_error_codes are
// matched at an offset.
// https://github.com/heimdal/MKShim/blob/964a930dfee5942efb8364ac07997ab5b2480033/Kerberos/krb5.h#L2457

#include <uapi/linux/ptrace.h>

struct key_t {
  char c[80];
};

// Section for function count probe: finish_process_as_req

BPF_HASH(counts_finish_process_as_req, struct key_t);

int count_finish_process_as_req(struct pt_regs *ctx) {

  u64 match_val = PT_REGS_PARM2(ctx);
  u64 zero = 0, *count;

  if (match_val == 0) {
    struct key_t key = {.c = "NONE"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638913) {
    struct key_t key = {.c = "NAME_EXP"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638914) {
    struct key_t key = {.c = "SERVICE_EXP"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638915) {
    struct key_t key = {.c = "BAD_PVNO"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638916) {
    struct key_t key = {.c = "C_OLD_MAST_KVNO"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638917) {
    struct key_t key = {.c = "S_OLD_MAST_KVNO"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638918) {
    struct key_t key = {.c = "C_PRINCIPAL_UNKNOWN"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638919) {
    struct key_t key = {.c = "S_PRINCIPAL_UNKNOWN"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638920) {
    struct key_t key = {.c = "PRINCIPAL_NOT_UNIQUE"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638921) {
    struct key_t key = {.c = "NULL_KEY"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638922) {
    struct key_t key = {.c = "CANNOT_POSTDATE"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638923) {
    struct key_t key = {.c = "NEVER_VALID"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638924) {
    struct key_t key = {.c = "POLICY"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638925) {
    struct key_t key = {.c = "BADOPTION"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638926) {
    struct key_t key = {.c = "ETYPE_NOSUPP"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638927) {
    struct key_t key = {.c = "SUMTYPE_NOSUPP"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638928) {
    struct key_t key = {.c = "PADATA_TYPE_NOSUPP"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638929) {
    struct key_t key = {.c = "TRTYPE_NOSUPP"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638930) {
    struct key_t key = {.c = "CLIENT_REVOKED"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638931) {
    struct key_t key = {.c = "SERVICE_REVOKED"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638932) {
    struct key_t key = {.c = "TGT_REVOKED"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638933) {
    struct key_t key = {.c = "CLIENT_NOTYET"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638934) {
    struct key_t key = {.c = "SERVICE_NOTYET"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638935) {
    struct key_t key = {.c = "KEY_EXP"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638936) {
    struct key_t key = {.c = "PREAUTH_FAILED"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638937) {
    struct key_t key = {.c = "PREAUTH_REQUIRED"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638938) {
    struct key_t key = {.c = "SERVER_NOMATCH"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638939) {
    struct key_t key = {.c = "MUST_USE_USER2USER"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638940) {
    struct key_t key = {.c = "PATH_NOT_ACCEPTED"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638941) {
    struct key_t key = {.c = "SVC_UNAVAILABLE"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  } else {
    struct key_t key = {.c = "UNKNOWN"};
    count = counts_finish_process_as_req.lookup_or_init(&key, &zero);
  }

  (*count)++;

  return 0;
}

// Section for function count probe: finish_dispatch_cache

BPF_HASH(counts_finish_dispatch_cache, struct key_t);

int count_finish_dispatch_cache(struct pt_regs *ctx) {

  u64 match_val = PT_REGS_PARM2(ctx);
  u64 zero = 0, *count;

  if (match_val == 0) {
    struct key_t key = {.c = "NONE"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638913) {
    struct key_t key = {.c = "NAME_EXP"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638914) {
    struct key_t key = {.c = "SERVICE_EXP"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638915) {
    struct key_t key = {.c = "BAD_PVNO"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638916) {
    struct key_t key = {.c = "C_OLD_MAST_KVNO"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638917) {
    struct key_t key = {.c = "S_OLD_MAST_KVNO"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638918) {
    struct key_t key = {.c = "C_PRINCIPAL_UNKNOWN"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638919) {
    struct key_t key = {.c = "S_PRINCIPAL_UNKNOWN"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638920) {
    struct key_t key = {.c = "PRINCIPAL_NOT_UNIQUE"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638921) {
    struct key_t key = {.c = "NULL_KEY"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638922) {
    struct key_t key = {.c = "CANNOT_POSTDATE"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638923) {
    struct key_t key = {.c = "NEVER_VALID"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638924) {
    struct key_t key = {.c = "POLICY"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638925) {
    struct key_t key = {.c = "BADOPTION"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638926) {
    struct key_t key = {.c = "ETYPE_NOSUPP"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638927) {
    struct key_t key = {.c = "SUMTYPE_NOSUPP"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638928) {
    struct key_t key = {.c = "PADATA_TYPE_NOSUPP"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638929) {
    struct key_t key = {.c = "TRTYPE_NOSUPP"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638930) {
    struct key_t key = {.c = "CLIENT_REVOKED"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638931) {
    struct key_t key = {.c = "SERVICE_REVOKED"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638932) {
    struct key_t key = {.c = "TGT_REVOKED"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638933) {
    struct key_t key = {.c = "CLIENT_NOTYET"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638934) {
    struct key_t key = {.c = "SERVICE_NOTYET"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638935) {
    struct key_t key = {.c = "KEY_EXP"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638936) {
    struct key_t key = {.c = "PREAUTH_FAILED"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638937) {
    struct key_t key = {.c = "PREAUTH_REQUIRED"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638938) {
    struct key_t key = {.c = "SERVER_NOMATCH"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638939) {
    struct key_t key = {.c = "MUST_USE_USER2USER"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638940) {
    struct key_t key = {.c = "PATH_NOT_ACCEPTED"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638941) {
    struct key_t key = {.c = "SVC_UNAVAILABLE"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  } else {
    struct key_t key = {.c = "UNKNOWN"};
    count = counts_finish_dispatch_cache.lookup_or_init(&key, &zero);
  }

  (*count)++;

  return 0;
}

// Section for function count probe: process_tgs_req

BPF_HASH(counts_process_tgs_req, struct key_t);

int count_process_tgs_req(struct pt_regs *ctx) {

  u64 match_val = PT_REGS_RC(ctx);
  u64 zero = 0, *count;

  if (match_val == 0) {
    struct key_t key = {.c = "NONE"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638913) {
    struct key_t key = {.c = "NAME_EXP"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638914) {
    struct key_t key = {.c = "SERVICE_EXP"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638915) {
    struct key_t key = {.c = "BAD_PVNO"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638916) {
    struct key_t key = {.c = "C_OLD_MAST_KVNO"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638917) {
    struct key_t key = {.c = "S_OLD_MAST_KVNO"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638918) {
    struct key_t key = {.c = "C_PRINCIPAL_UNKNOWN"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638919) {
    struct key_t key = {.c = "S_PRINCIPAL_UNKNOWN"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638920) {
    struct key_t key = {.c = "PRINCIPAL_NOT_UNIQUE"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638921) {
    struct key_t key = {.c = "NULL_KEY"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638922) {
    struct key_t key = {.c = "CANNOT_POSTDATE"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638923) {
    struct key_t key = {.c = "NEVER_VALID"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638924) {
    struct key_t key = {.c = "POLICY"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638925) {
    struct key_t key = {.c = "BADOPTION"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638926) {
    struct key_t key = {.c = "ETYPE_NOSUPP"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638927) {
    struct key_t key = {.c = "SUMTYPE_NOSUPP"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638928) {
    struct key_t key = {.c = "PADATA_TYPE_NOSUPP"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638929) {
    struct key_t key = {.c = "TRTYPE_NOSUPP"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638930) {
    struct key_t key = {.c = "CLIENT_REVOKED"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638931) {
    struct key_t key = {.c = "SERVICE_REVOKED"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638932) {
    struct key_t key = {.c = "TGT_REVOKED"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638933) {
    struct key_t key = {.c = "CLIENT_NOTYET"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638934) {
    struct key_t key = {.c = "SERVICE_NOTYET"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638935) {
    struct key_t key = {.c = "KEY_EXP"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638936) {
    struct key_t key = {.c = "PREAUTH_FAILED"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638937) {
    struct key_t key = {.c = "PREAUTH_REQUIRED"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638938) {
    struct key_t key = {.c = "SERVER_NOMATCH"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638939) {
    struct key_t key = {.c = "MUST_USE_USER2USER"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638940) {
    struct key_t key = {.c = "PATH_NOT_ACCEPTED"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else if (match_val == 2529638941) {
    struct key_t key = {.c = "SVC_UNAVAILABLE"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  } else {
    struct key_t key = {.c = "UNKNOWN"};
    count = counts_process_tgs_req.lookup_or_init(&key, &zero);
  }

  (*count)++;

  return 0;
}
