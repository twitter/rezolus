// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::metrics::*;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

#[cfg(feature = "bpf")]
use crate::common::bpf::*;

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    EnumIter,
    EnumString,
    Eq,
    IntoStaticStr,
    PartialEq,
    Hash,
    Serialize,
)]
#[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
pub enum Krb5kdcStatistic {
    #[strum(serialize = "krb5kdc/finish_process_as_req/unknown")]
    FinishProcessAsReqUnknown,

    #[strum(serialize = "krb5kdc/finish_process_as_req/none")]
    FinishProcessAsReqNone,

    #[strum(serialize = "krb5kdc/finish_process_as_req/name_exp")]
    FinishProcessAsReqNameExp,

    #[strum(serialize = "krb5kdc/finish_process_as_req/service_exp")]
    FinishProcessAsReqServiceExp,

    #[strum(serialize = "krb5kdc/finish_process_as_req/bad_pvno")]
    FinishProcessAsReqBadPvno,

    #[strum(serialize = "krb5kdc/finish_process_as_req/c_old_mast_kvno")]
    FinishProcessAsReqCOldMastKvno,

    #[strum(serialize = "krb5kdc/finish_process_as_req/s_old_mast_kvno")]
    FinishProcessAsReqSOldMastKvno,

    #[strum(serialize = "krb5kdc/finish_process_as_req/c_principal_unknown")]
    FinishProcessAsReqCPrincipalUnknown,

    #[strum(serialize = "krb5kdc/finish_process_as_req/s_principal_unknown")]
    FinishProcessAsReqSPrincipalUnknown,

    #[strum(serialize = "krb5kdc/finish_process_as_req/principal_not_unique")]
    FinishProcessAsReqPrincipalNotUnique,

    #[strum(serialize = "krb5kdc/finish_process_as_req/null_key")]
    FinishProcessAsReqNullKey,

    #[strum(serialize = "krb5kdc/finish_process_as_req/cannot_postdate")]
    FinishProcessAsReqCannotPostdate,

    #[strum(serialize = "krb5kdc/finish_process_as_req/never_valid")]
    FinishProcessAsReqNeverValid,

    #[strum(serialize = "krb5kdc/finish_process_as_req/policy")]
    FinishProcessAsReqPolicy,

    #[strum(serialize = "krb5kdc/finish_process_as_req/badoption")]
    FinishProcessAsReqBadoption,

    #[strum(serialize = "krb5kdc/finish_process_as_req/etype_nosupp")]
    FinishProcessAsReqEtypeNosupp,

    #[strum(serialize = "krb5kdc/finish_process_as_req/sumtype_nosupp")]
    FinishProcessAsReqSumtypeNosupp,

    #[strum(serialize = "krb5kdc/finish_process_as_req/padata_type_nosupp")]
    FinishProcessAsReqPadataTypeNosupp,

    #[strum(serialize = "krb5kdc/finish_process_as_req/trtype_nosupp")]
    FinishProcessAsReqTrtypeNosupp,

    #[strum(serialize = "krb5kdc/finish_process_as_req/client_revoked")]
    FinishProcessAsReqClientRevoked,

    #[strum(serialize = "krb5kdc/finish_process_as_req/service_revoked")]
    FinishProcessAsReqServiceRevoked,

    #[strum(serialize = "krb5kdc/finish_process_as_req/tgt_revoked")]
    FinishProcessAsReqTgtRevoked,

    #[strum(serialize = "krb5kdc/finish_process_as_req/client_notyet")]
    FinishProcessAsReqClientNotyet,

    #[strum(serialize = "krb5kdc/finish_process_as_req/service_notyet")]
    FinishProcessAsReqServiceNotyet,

    #[strum(serialize = "krb5kdc/finish_process_as_req/key_exp")]
    FinishProcessAsReqKeyExp,

    #[strum(serialize = "krb5kdc/finish_process_as_req/preauth_failed")]
    FinishProcessAsReqPreauthFailed,

    #[strum(serialize = "krb5kdc/finish_process_as_req/preauth_required")]
    FinishProcessAsReqPreauthRequired,

    #[strum(serialize = "krb5kdc/finish_process_as_req/server_nomatch")]
    FinishProcessAsReqServerNomatch,

    #[strum(serialize = "krb5kdc/finish_process_as_req/must_use_user2user")]
    FinishProcessAsReqMustUseUser2user,

    #[strum(serialize = "krb5kdc/finish_process_as_req/path_not_accepted")]
    FinishProcessAsReqPathNotAccepted,

    #[strum(serialize = "krb5kdc/finish_process_as_req/svc_unavailable")]
    FinishProcessAsReqSvcUnavailable,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/unknown")]
    FinishDispatchCacheUnknown,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/none")]
    FinishDispatchCacheNone,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/name_exp")]
    FinishDispatchCacheNameExp,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/service_exp")]
    FinishDispatchCacheServiceExp,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/bad_pvno")]
    FinishDispatchCacheBadPvno,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/c_old_mast_kvno")]
    FinishDispatchCacheCOldMastKvno,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/s_old_mast_kvno")]
    FinishDispatchCacheSOldMastKvno,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/c_principal_unknown")]
    FinishDispatchCacheCPrincipalUnknown,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/s_principal_unknown")]
    FinishDispatchCacheSPrincipalUnknown,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/principal_not_unique")]
    FinishDispatchCachePrincipalNotUnique,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/null_key")]
    FinishDispatchCacheNullKey,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/cannot_postdate")]
    FinishDispatchCacheCannotPostdate,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/never_valid")]
    FinishDispatchCacheNeverValid,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/policy")]
    FinishDispatchCachePolicy,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/badoption")]
    FinishDispatchCacheBadoption,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/etype_nosupp")]
    FinishDispatchCacheEtypeNosupp,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/sumtype_nosupp")]
    FinishDispatchCacheSumtypeNosupp,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/padata_type_nosupp")]
    FinishDispatchCachePadataTypeNosupp,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/trtype_nosupp")]
    FinishDispatchCacheTrtypeNosupp,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/client_revoked")]
    FinishDispatchCacheClientRevoked,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/service_revoked")]
    FinishDispatchCacheServiceRevoked,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/tgt_revoked")]
    FinishDispatchCacheTgtRevoked,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/client_notyet")]
    FinishDispatchCacheClientNotyet,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/service_notyet")]
    FinishDispatchCacheServiceNotyet,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/key_exp")]
    FinishDispatchCacheKeyExp,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/preauth_failed")]
    FinishDispatchCachePreauthFailed,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/preauth_required")]
    FinishDispatchCachePreauthRequired,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/server_nomatch")]
    FinishDispatchCacheServerNomatch,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/must_use_user2user")]
    FinishDispatchCacheMustUseUser2user,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/path_not_accepted")]
    FinishDispatchCachePathNotAccepted,

    #[strum(serialize = "krb5kdc/finish_dispatch_cache/svc_unavailable")]
    FinishDispatchCacheSvcUnavailable,

    #[strum(serialize = "krb5kdc/process_tgs_req/unknown")]
    ProcessTgsReqUnknown,

    #[strum(serialize = "krb5kdc/process_tgs_req/none")]
    ProcessTgsReqNone,

    #[strum(serialize = "krb5kdc/process_tgs_req/name_exp")]
    ProcessTgsReqNameExp,

    #[strum(serialize = "krb5kdc/process_tgs_req/service_exp")]
    ProcessTgsReqServiceExp,

    #[strum(serialize = "krb5kdc/process_tgs_req/bad_pvno")]
    ProcessTgsReqBadPvno,

    #[strum(serialize = "krb5kdc/process_tgs_req/c_old_mast_kvno")]
    ProcessTgsReqCOldMastKvno,

    #[strum(serialize = "krb5kdc/process_tgs_req/s_old_mast_kvno")]
    ProcessTgsReqSOldMastKvno,

    #[strum(serialize = "krb5kdc/process_tgs_req/c_principal_unknown")]
    ProcessTgsReqCPrincipalUnknown,

    #[strum(serialize = "krb5kdc/process_tgs_req/s_principal_unknown")]
    ProcessTgsReqSPrincipalUnknown,

    #[strum(serialize = "krb5kdc/process_tgs_req/principal_not_unique")]
    ProcessTgsReqPrincipalNotUnique,

    #[strum(serialize = "krb5kdc/process_tgs_req/null_key")]
    ProcessTgsReqNullKey,

    #[strum(serialize = "krb5kdc/process_tgs_req/cannot_postdate")]
    ProcessTgsReqCannotPostdate,

    #[strum(serialize = "krb5kdc/process_tgs_req/never_valid")]
    ProcessTgsReqNeverValid,

    #[strum(serialize = "krb5kdc/process_tgs_req/policy")]
    ProcessTgsReqPolicy,

    #[strum(serialize = "krb5kdc/process_tgs_req/badoption")]
    ProcessTgsReqBadoption,

    #[strum(serialize = "krb5kdc/process_tgs_req/etype_nosupp")]
    ProcessTgsReqEtypeNosupp,

    #[strum(serialize = "krb5kdc/process_tgs_req/sumtype_nosupp")]
    ProcessTgsReqSumtypeNosupp,

    #[strum(serialize = "krb5kdc/process_tgs_req/padata_type_nosupp")]
    ProcessTgsReqPadataTypeNosupp,

    #[strum(serialize = "krb5kdc/process_tgs_req/trtype_nosupp")]
    ProcessTgsReqTrtypeNosupp,

    #[strum(serialize = "krb5kdc/process_tgs_req/client_revoked")]
    ProcessTgsReqClientRevoked,

    #[strum(serialize = "krb5kdc/process_tgs_req/service_revoked")]
    ProcessTgsReqServiceRevoked,

    #[strum(serialize = "krb5kdc/process_tgs_req/tgt_revoked")]
    ProcessTgsReqTgtRevoked,

    #[strum(serialize = "krb5kdc/process_tgs_req/client_notyet")]
    ProcessTgsReqClientNotyet,

    #[strum(serialize = "krb5kdc/process_tgs_req/service_notyet")]
    ProcessTgsReqServiceNotyet,

    #[strum(serialize = "krb5kdc/process_tgs_req/key_exp")]
    ProcessTgsReqKeyExp,

    #[strum(serialize = "krb5kdc/process_tgs_req/preauth_failed")]
    ProcessTgsReqPreauthFailed,

    #[strum(serialize = "krb5kdc/process_tgs_req/preauth_required")]
    ProcessTgsReqPreauthRequired,

    #[strum(serialize = "krb5kdc/process_tgs_req/server_nomatch")]
    ProcessTgsReqServerNomatch,

    #[strum(serialize = "krb5kdc/process_tgs_req/must_use_user2user")]
    ProcessTgsReqMustUseUser2user,

    #[strum(serialize = "krb5kdc/process_tgs_req/path_not_accepted")]
    ProcessTgsReqPathNotAccepted,

    #[strum(serialize = "krb5kdc/process_tgs_req/svc_unavailable")]
    ProcessTgsReqSvcUnavailable,
}

impl Krb5kdcStatistic {
    pub fn bpf_table(self) -> &'static str {
        match self {
            Self::FinishProcessAsReqUnknown => "counts_finish_process_as_req",
            Self::FinishProcessAsReqNone => "counts_finish_process_as_req",
            Self::FinishProcessAsReqNameExp => "counts_finish_process_as_req",
            Self::FinishProcessAsReqServiceExp => "counts_finish_process_as_req",
            Self::FinishProcessAsReqBadPvno => "counts_finish_process_as_req",
            Self::FinishProcessAsReqCOldMastKvno => "counts_finish_process_as_req",
            Self::FinishProcessAsReqSOldMastKvno => "counts_finish_process_as_req",
            Self::FinishProcessAsReqCPrincipalUnknown => "counts_finish_process_as_req",
            Self::FinishProcessAsReqSPrincipalUnknown => "counts_finish_process_as_req",
            Self::FinishProcessAsReqPrincipalNotUnique => "counts_finish_process_as_req",
            Self::FinishProcessAsReqNullKey => "counts_finish_process_as_req",
            Self::FinishProcessAsReqCannotPostdate => "counts_finish_process_as_req",
            Self::FinishProcessAsReqNeverValid => "counts_finish_process_as_req",
            Self::FinishProcessAsReqPolicy => "counts_finish_process_as_req",
            Self::FinishProcessAsReqBadoption => "counts_finish_process_as_req",
            Self::FinishProcessAsReqEtypeNosupp => "counts_finish_process_as_req",
            Self::FinishProcessAsReqSumtypeNosupp => "counts_finish_process_as_req",
            Self::FinishProcessAsReqPadataTypeNosupp => "counts_finish_process_as_req",
            Self::FinishProcessAsReqTrtypeNosupp => "counts_finish_process_as_req",
            Self::FinishProcessAsReqClientRevoked => "counts_finish_process_as_req",
            Self::FinishProcessAsReqServiceRevoked => "counts_finish_process_as_req",
            Self::FinishProcessAsReqTgtRevoked => "counts_finish_process_as_req",
            Self::FinishProcessAsReqClientNotyet => "counts_finish_process_as_req",
            Self::FinishProcessAsReqServiceNotyet => "counts_finish_process_as_req",
            Self::FinishProcessAsReqKeyExp => "counts_finish_process_as_req",
            Self::FinishProcessAsReqPreauthFailed => "counts_finish_process_as_req",
            Self::FinishProcessAsReqPreauthRequired => "counts_finish_process_as_req",
            Self::FinishProcessAsReqServerNomatch => "counts_finish_process_as_req",
            Self::FinishProcessAsReqMustUseUser2user => "counts_finish_process_as_req",
            Self::FinishProcessAsReqPathNotAccepted => "counts_finish_process_as_req",
            Self::FinishProcessAsReqSvcUnavailable => "counts_finish_process_as_req",

            Self::FinishDispatchCacheUnknown => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheNone => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheNameExp => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheServiceExp => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheBadPvno => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheCOldMastKvno => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheSOldMastKvno => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheCPrincipalUnknown => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheSPrincipalUnknown => "counts_finish_dispatch_cache",
            Self::FinishDispatchCachePrincipalNotUnique => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheNullKey => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheCannotPostdate => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheNeverValid => "counts_finish_dispatch_cache",
            Self::FinishDispatchCachePolicy => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheBadoption => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheEtypeNosupp => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheSumtypeNosupp => "counts_finish_dispatch_cache",
            Self::FinishDispatchCachePadataTypeNosupp => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheTrtypeNosupp => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheClientRevoked => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheServiceRevoked => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheTgtRevoked => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheClientNotyet => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheServiceNotyet => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheKeyExp => "counts_finish_dispatch_cache",
            Self::FinishDispatchCachePreauthFailed => "counts_finish_dispatch_cache",
            Self::FinishDispatchCachePreauthRequired => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheServerNomatch => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheMustUseUser2user => "counts_finish_dispatch_cache",
            Self::FinishDispatchCachePathNotAccepted => "counts_finish_dispatch_cache",
            Self::FinishDispatchCacheSvcUnavailable => "counts_finish_dispatch_cache",

            Self::ProcessTgsReqUnknown => "counts_process_tgs_req",
            Self::ProcessTgsReqNone => "counts_process_tgs_req",
            Self::ProcessTgsReqNameExp => "counts_process_tgs_req",
            Self::ProcessTgsReqServiceExp => "counts_process_tgs_req",
            Self::ProcessTgsReqBadPvno => "counts_process_tgs_req",
            Self::ProcessTgsReqCOldMastKvno => "counts_process_tgs_req",
            Self::ProcessTgsReqSOldMastKvno => "counts_process_tgs_req",
            Self::ProcessTgsReqCPrincipalUnknown => "counts_process_tgs_req",
            Self::ProcessTgsReqSPrincipalUnknown => "counts_process_tgs_req",
            Self::ProcessTgsReqPrincipalNotUnique => "counts_process_tgs_req",
            Self::ProcessTgsReqNullKey => "counts_process_tgs_req",
            Self::ProcessTgsReqCannotPostdate => "counts_process_tgs_req",
            Self::ProcessTgsReqNeverValid => "counts_process_tgs_req",
            Self::ProcessTgsReqPolicy => "counts_process_tgs_req",
            Self::ProcessTgsReqBadoption => "counts_process_tgs_req",
            Self::ProcessTgsReqEtypeNosupp => "counts_process_tgs_req",
            Self::ProcessTgsReqSumtypeNosupp => "counts_process_tgs_req",
            Self::ProcessTgsReqPadataTypeNosupp => "counts_process_tgs_req",
            Self::ProcessTgsReqTrtypeNosupp => "counts_process_tgs_req",
            Self::ProcessTgsReqClientRevoked => "counts_process_tgs_req",
            Self::ProcessTgsReqServiceRevoked => "counts_process_tgs_req",
            Self::ProcessTgsReqTgtRevoked => "counts_process_tgs_req",
            Self::ProcessTgsReqClientNotyet => "counts_process_tgs_req",
            Self::ProcessTgsReqServiceNotyet => "counts_process_tgs_req",
            Self::ProcessTgsReqKeyExp => "counts_process_tgs_req",
            Self::ProcessTgsReqPreauthFailed => "counts_process_tgs_req",
            Self::ProcessTgsReqPreauthRequired => "counts_process_tgs_req",
            Self::ProcessTgsReqServerNomatch => "counts_process_tgs_req",
            Self::ProcessTgsReqMustUseUser2user => "counts_process_tgs_req",
            Self::ProcessTgsReqPathNotAccepted => "counts_process_tgs_req",
            Self::ProcessTgsReqSvcUnavailable => "counts_process_tgs_req",
        }
    }

    pub fn bpf_entry(self) -> &'static str {
        match self {
            Self::FinishProcessAsReqUnknown => "UNKNOWN",
            Self::FinishProcessAsReqNone => "NONE",
            Self::FinishProcessAsReqNameExp => "NAME_EXP",
            Self::FinishProcessAsReqServiceExp => "SERVICE_EXP",
            Self::FinishProcessAsReqBadPvno => "BAD_PVNO",
            Self::FinishProcessAsReqCOldMastKvno => "C_OLD_MAST_KVNO",
            Self::FinishProcessAsReqSOldMastKvno => "S_OLD_MAST_KVNO",
            Self::FinishProcessAsReqCPrincipalUnknown => "C_PRINCIPAL_UNKNOWN",
            Self::FinishProcessAsReqSPrincipalUnknown => "S_PRINCIPAL_UNKNOWN",
            Self::FinishProcessAsReqPrincipalNotUnique => "PRINCIPAL_NOT_UNIQUE",
            Self::FinishProcessAsReqNullKey => "NULL_KEY",
            Self::FinishProcessAsReqCannotPostdate => "CANNOT_POSTDATE",
            Self::FinishProcessAsReqNeverValid => "NEVER_VALID",
            Self::FinishProcessAsReqPolicy => "POLICY",
            Self::FinishProcessAsReqBadoption => "BADOPTION",
            Self::FinishProcessAsReqEtypeNosupp => "ETYPE_NOSUPP",
            Self::FinishProcessAsReqSumtypeNosupp => "SUMTYPE_NOSUPP",
            Self::FinishProcessAsReqPadataTypeNosupp => "PADATA_TYPE_NOSUPP",
            Self::FinishProcessAsReqTrtypeNosupp => "TRTYPE_NOSUPP",
            Self::FinishProcessAsReqClientRevoked => "CLIENT_REVOKED",
            Self::FinishProcessAsReqServiceRevoked => "SERVICE_REVOKED",
            Self::FinishProcessAsReqTgtRevoked => "TGT_REVOKED",
            Self::FinishProcessAsReqClientNotyet => "CLIENT_NOTYET",
            Self::FinishProcessAsReqServiceNotyet => "SERVICE_NOTYET",
            Self::FinishProcessAsReqKeyExp => "KEY_EXP",
            Self::FinishProcessAsReqPreauthFailed => "PREAUTH_FAILED",
            Self::FinishProcessAsReqPreauthRequired => "PREAUTH_REQUIRED",
            Self::FinishProcessAsReqServerNomatch => "SERVER_NOMATCH",
            Self::FinishProcessAsReqMustUseUser2user => "MUST_USE_USER2USER",
            Self::FinishProcessAsReqPathNotAccepted => "PATH_NOT_ACCEPTED",
            Self::FinishProcessAsReqSvcUnavailable => "SVC_UNAVAILABLE",

            Self::FinishDispatchCacheUnknown => "UNKNOWN",
            Self::FinishDispatchCacheNone => "NONE",
            Self::FinishDispatchCacheNameExp => "NAME_EXP",
            Self::FinishDispatchCacheServiceExp => "SERVICE_EXP",
            Self::FinishDispatchCacheBadPvno => "BAD_PVNO",
            Self::FinishDispatchCacheCOldMastKvno => "C_OLD_MAST_KVNO",
            Self::FinishDispatchCacheSOldMastKvno => "S_OLD_MAST_KVNO",
            Self::FinishDispatchCacheCPrincipalUnknown => "C_PRINCIPAL_UNKNOWN",
            Self::FinishDispatchCacheSPrincipalUnknown => "S_PRINCIPAL_UNKNOWN",
            Self::FinishDispatchCachePrincipalNotUnique => "PRINCIPAL_NOT_UNIQUE",
            Self::FinishDispatchCacheNullKey => "NULL_KEY",
            Self::FinishDispatchCacheCannotPostdate => "CANNOT_POSTDATE",
            Self::FinishDispatchCacheNeverValid => "NEVER_VALID",
            Self::FinishDispatchCachePolicy => "POLICY",
            Self::FinishDispatchCacheBadoption => "BADOPTION",
            Self::FinishDispatchCacheEtypeNosupp => "ETYPE_NOSUPP",
            Self::FinishDispatchCacheSumtypeNosupp => "SUMTYPE_NOSUPP",
            Self::FinishDispatchCachePadataTypeNosupp => "PADATA_TYPE_NOSUPP",
            Self::FinishDispatchCacheTrtypeNosupp => "TRTYPE_NOSUPP",
            Self::FinishDispatchCacheClientRevoked => "CLIENT_REVOKED",
            Self::FinishDispatchCacheServiceRevoked => "SERVICE_REVOKED",
            Self::FinishDispatchCacheTgtRevoked => "TGT_REVOKED",
            Self::FinishDispatchCacheClientNotyet => "CLIENT_NOTYET",
            Self::FinishDispatchCacheServiceNotyet => "SERVICE_NOTYET",
            Self::FinishDispatchCacheKeyExp => "KEY_EXP",
            Self::FinishDispatchCachePreauthFailed => "PREAUTH_FAILED",
            Self::FinishDispatchCachePreauthRequired => "PREAUTH_REQUIRED",
            Self::FinishDispatchCacheServerNomatch => "SERVER_NOMATCH",
            Self::FinishDispatchCacheMustUseUser2user => "MUST_USE_USER2USER",
            Self::FinishDispatchCachePathNotAccepted => "PATH_NOT_ACCEPTED",
            Self::FinishDispatchCacheSvcUnavailable => "SVC_UNAVAILABLE",

            Self::ProcessTgsReqUnknown => "UNKNOWN",
            Self::ProcessTgsReqNone => "NONE",
            Self::ProcessTgsReqNameExp => "NAME_EXP",
            Self::ProcessTgsReqServiceExp => "SERVICE_EXP",
            Self::ProcessTgsReqBadPvno => "BAD_PVNO",
            Self::ProcessTgsReqCOldMastKvno => "C_OLD_MAST_KVNO",
            Self::ProcessTgsReqSOldMastKvno => "S_OLD_MAST_KVNO",
            Self::ProcessTgsReqCPrincipalUnknown => "C_PRINCIPAL_UNKNOWN",
            Self::ProcessTgsReqSPrincipalUnknown => "S_PRINCIPAL_UNKNOWN",
            Self::ProcessTgsReqPrincipalNotUnique => "PRINCIPAL_NOT_UNIQUE",
            Self::ProcessTgsReqNullKey => "NULL_KEY",
            Self::ProcessTgsReqCannotPostdate => "CANNOT_POSTDATE",
            Self::ProcessTgsReqNeverValid => "NEVER_VALID",
            Self::ProcessTgsReqPolicy => "POLICY",
            Self::ProcessTgsReqBadoption => "BADOPTION",
            Self::ProcessTgsReqEtypeNosupp => "ETYPE_NOSUPP",
            Self::ProcessTgsReqSumtypeNosupp => "SUMTYPE_NOSUPP",
            Self::ProcessTgsReqPadataTypeNosupp => "PADATA_TYPE_NOSUPP",
            Self::ProcessTgsReqTrtypeNosupp => "TRTYPE_NOSUPP",
            Self::ProcessTgsReqClientRevoked => "CLIENT_REVOKED",
            Self::ProcessTgsReqServiceRevoked => "SERVICE_REVOKED",
            Self::ProcessTgsReqTgtRevoked => "TGT_REVOKED",
            Self::ProcessTgsReqClientNotyet => "CLIENT_NOTYET",
            Self::ProcessTgsReqServiceNotyet => "SERVICE_NOTYET",
            Self::ProcessTgsReqKeyExp => "KEY_EXP",
            Self::ProcessTgsReqPreauthFailed => "PREAUTH_FAILED",
            Self::ProcessTgsReqPreauthRequired => "PREAUTH_REQUIRED",
            Self::ProcessTgsReqServerNomatch => "SERVER_NOMATCH",
            Self::ProcessTgsReqMustUseUser2user => "MUST_USE_USER2USER",
            Self::ProcessTgsReqPathNotAccepted => "PATH_NOT_ACCEPTED",
            Self::ProcessTgsReqSvcUnavailable => "SVC_UNAVAILABLE",
        }
    }

    #[cfg(feature = "bpf")]
    pub fn bpf_probes_required(self, binary_path: String) -> Vec<Probe> {
        // define the unique probes below.
        let process_probe = Probe {
            name: "finish_process_as_req".to_string(),
            handler: "count_finish_process_as_req".to_string(),
            probe_type: ProbeType::User,
            probe_location: ProbeLocation::Entry,
            binary_path: Some(binary_path.clone()),
            sub_system: None,
        };
        let dispatch_probe = Probe {
            name: "finish_dispatch_cache".to_string(),
            handler: "count_finish_dispatch_cache".to_string(),
            probe_type: ProbeType::User,
            probe_location: ProbeLocation::Entry,
            binary_path: Some(binary_path.clone()),
            sub_system: None,
        };
        let tgs_req_probe = Probe {
            name: "process_tgs_req".to_string(),
            handler: "count_process_tgs_req".to_string(),
            probe_type: ProbeType::User,
            probe_location: ProbeLocation::Return,
            binary_path: Some(binary_path.clone()),
            sub_system: None,
        };
        match self {
            Self::FinishProcessAsReqUnknown => vec![process_probe],
            Self::FinishProcessAsReqNone => vec![process_probe],
            Self::FinishProcessAsReqNameExp => vec![process_probe],
            Self::FinishProcessAsReqServiceExp => vec![process_probe],
            Self::FinishProcessAsReqBadPvno => vec![process_probe],
            Self::FinishProcessAsReqCOldMastKvno => vec![process_probe],
            Self::FinishProcessAsReqSOldMastKvno => vec![process_probe],
            Self::FinishProcessAsReqCPrincipalUnknown => vec![process_probe],
            Self::FinishProcessAsReqSPrincipalUnknown => vec![process_probe],
            Self::FinishProcessAsReqPrincipalNotUnique => vec![process_probe],
            Self::FinishProcessAsReqNullKey => vec![process_probe],
            Self::FinishProcessAsReqCannotPostdate => vec![process_probe],
            Self::FinishProcessAsReqNeverValid => vec![process_probe],
            Self::FinishProcessAsReqPolicy => vec![process_probe],
            Self::FinishProcessAsReqBadoption => vec![process_probe],
            Self::FinishProcessAsReqEtypeNosupp => vec![process_probe],
            Self::FinishProcessAsReqSumtypeNosupp => vec![process_probe],
            Self::FinishProcessAsReqPadataTypeNosupp => vec![process_probe],
            Self::FinishProcessAsReqTrtypeNosupp => vec![process_probe],
            Self::FinishProcessAsReqClientRevoked => vec![process_probe],
            Self::FinishProcessAsReqServiceRevoked => vec![process_probe],
            Self::FinishProcessAsReqTgtRevoked => vec![process_probe],
            Self::FinishProcessAsReqClientNotyet => vec![process_probe],
            Self::FinishProcessAsReqServiceNotyet => vec![process_probe],
            Self::FinishProcessAsReqKeyExp => vec![process_probe],
            Self::FinishProcessAsReqPreauthFailed => vec![process_probe],
            Self::FinishProcessAsReqPreauthRequired => vec![process_probe],
            Self::FinishProcessAsReqServerNomatch => vec![process_probe],
            Self::FinishProcessAsReqMustUseUser2user => vec![process_probe],
            Self::FinishProcessAsReqPathNotAccepted => vec![process_probe],
            Self::FinishProcessAsReqSvcUnavailable => vec![process_probe],

            Self::FinishDispatchCacheUnknown => vec![dispatch_probe],
            Self::FinishDispatchCacheNone => vec![dispatch_probe],
            Self::FinishDispatchCacheNameExp => vec![dispatch_probe],
            Self::FinishDispatchCacheServiceExp => vec![dispatch_probe],
            Self::FinishDispatchCacheBadPvno => vec![dispatch_probe],
            Self::FinishDispatchCacheCOldMastKvno => vec![dispatch_probe],
            Self::FinishDispatchCacheSOldMastKvno => vec![dispatch_probe],
            Self::FinishDispatchCacheCPrincipalUnknown => vec![dispatch_probe],
            Self::FinishDispatchCacheSPrincipalUnknown => vec![dispatch_probe],
            Self::FinishDispatchCachePrincipalNotUnique => vec![dispatch_probe],
            Self::FinishDispatchCacheNullKey => vec![dispatch_probe],
            Self::FinishDispatchCacheCannotPostdate => vec![dispatch_probe],
            Self::FinishDispatchCacheNeverValid => vec![dispatch_probe],
            Self::FinishDispatchCachePolicy => vec![dispatch_probe],
            Self::FinishDispatchCacheBadoption => vec![dispatch_probe],
            Self::FinishDispatchCacheEtypeNosupp => vec![dispatch_probe],
            Self::FinishDispatchCacheSumtypeNosupp => vec![dispatch_probe],
            Self::FinishDispatchCachePadataTypeNosupp => vec![dispatch_probe],
            Self::FinishDispatchCacheTrtypeNosupp => vec![dispatch_probe],
            Self::FinishDispatchCacheClientRevoked => vec![dispatch_probe],
            Self::FinishDispatchCacheServiceRevoked => vec![dispatch_probe],
            Self::FinishDispatchCacheTgtRevoked => vec![dispatch_probe],
            Self::FinishDispatchCacheClientNotyet => vec![dispatch_probe],
            Self::FinishDispatchCacheServiceNotyet => vec![dispatch_probe],
            Self::FinishDispatchCacheKeyExp => vec![dispatch_probe],
            Self::FinishDispatchCachePreauthFailed => vec![dispatch_probe],
            Self::FinishDispatchCachePreauthRequired => vec![dispatch_probe],
            Self::FinishDispatchCacheServerNomatch => vec![dispatch_probe],
            Self::FinishDispatchCacheMustUseUser2user => vec![dispatch_probe],
            Self::FinishDispatchCachePathNotAccepted => vec![dispatch_probe],
            Self::FinishDispatchCacheSvcUnavailable => vec![dispatch_probe],

            Self::ProcessTgsReqUnknown => vec![tgs_req_probe],
            Self::ProcessTgsReqNone => vec![tgs_req_probe],
            Self::ProcessTgsReqNameExp => vec![tgs_req_probe],
            Self::ProcessTgsReqServiceExp => vec![tgs_req_probe],
            Self::ProcessTgsReqBadPvno => vec![tgs_req_probe],
            Self::ProcessTgsReqCOldMastKvno => vec![tgs_req_probe],
            Self::ProcessTgsReqSOldMastKvno => vec![tgs_req_probe],
            Self::ProcessTgsReqCPrincipalUnknown => vec![tgs_req_probe],
            Self::ProcessTgsReqSPrincipalUnknown => vec![tgs_req_probe],
            Self::ProcessTgsReqPrincipalNotUnique => vec![tgs_req_probe],
            Self::ProcessTgsReqNullKey => vec![tgs_req_probe],
            Self::ProcessTgsReqCannotPostdate => vec![tgs_req_probe],
            Self::ProcessTgsReqNeverValid => vec![tgs_req_probe],
            Self::ProcessTgsReqPolicy => vec![tgs_req_probe],
            Self::ProcessTgsReqBadoption => vec![tgs_req_probe],
            Self::ProcessTgsReqEtypeNosupp => vec![tgs_req_probe],
            Self::ProcessTgsReqSumtypeNosupp => vec![tgs_req_probe],
            Self::ProcessTgsReqPadataTypeNosupp => vec![tgs_req_probe],
            Self::ProcessTgsReqTrtypeNosupp => vec![tgs_req_probe],
            Self::ProcessTgsReqClientRevoked => vec![tgs_req_probe],
            Self::ProcessTgsReqServiceRevoked => vec![tgs_req_probe],
            Self::ProcessTgsReqTgtRevoked => vec![tgs_req_probe],
            Self::ProcessTgsReqClientNotyet => vec![tgs_req_probe],
            Self::ProcessTgsReqServiceNotyet => vec![tgs_req_probe],
            Self::ProcessTgsReqKeyExp => vec![tgs_req_probe],
            Self::ProcessTgsReqPreauthFailed => vec![tgs_req_probe],
            Self::ProcessTgsReqPreauthRequired => vec![tgs_req_probe],
            Self::ProcessTgsReqServerNomatch => vec![tgs_req_probe],
            Self::ProcessTgsReqMustUseUser2user => vec![tgs_req_probe],
            Self::ProcessTgsReqPathNotAccepted => vec![tgs_req_probe],
            Self::ProcessTgsReqSvcUnavailable => vec![tgs_req_probe],
        }
    }
}

impl Statistic for Krb5kdcStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        Source::Counter
    }
}
