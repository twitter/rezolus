// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

use rustcommon_metrics::*;
use serde_derive::{Deserialize, Serialize};
use strum::ParseError;
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
    pub fn bpf_probes_required(self, binary_path: String) -> Vec<FunctionProbe> {
        // define the unique probes below.
        let process_probe = FunctionProbe {
            name: String::from("finish_process_as_req"),
            handler: String::from("count_finish_process_as_req"),
            probe_type: ProbeType::User,
            probe_location: ProbeLocation::Entry,
            binary_path: Some(binary_path.clone()),
            sub_system: None,
        };
        let dispatch_probe = FunctionProbe {
            name: String::from("finish_dispatch_cache"),
            handler: String::from("count_finish_dispatch_cache"),
            probe_type: ProbeType::User,
            probe_location: ProbeLocation::Entry,
            binary_path: Some(binary_path.clone()),
            sub_system: None,
        };
        let tgs_req_probe = FunctionProbe {
            name: String::from("process_tgs_req"),
            handler: String::from("count_process_tgs_req"),
            probe_type: ProbeType::User,
            probe_location: ProbeLocation::Return,
            binary_path: Some(binary_path.clone()),
            sub_system: None,
        };
        match self {
            Self::FinishProcessAsReqUnknown => [process_probe].to_vec(),
            Self::FinishProcessAsReqNone => [process_probe].to_vec(),
            Self::FinishProcessAsReqNameExp => [process_probe].to_vec(),
            Self::FinishProcessAsReqServiceExp => [process_probe].to_vec(),
            Self::FinishProcessAsReqBadPvno => [process_probe].to_vec(),
            Self::FinishProcessAsReqCOldMastKvno => [process_probe].to_vec(),
            Self::FinishProcessAsReqSOldMastKvno => [process_probe].to_vec(),
            Self::FinishProcessAsReqCPrincipalUnknown => [process_probe].to_vec(),
            Self::FinishProcessAsReqSPrincipalUnknown => [process_probe].to_vec(),
            Self::FinishProcessAsReqPrincipalNotUnique => [process_probe].to_vec(),
            Self::FinishProcessAsReqNullKey => [process_probe].to_vec(),
            Self::FinishProcessAsReqCannotPostdate => [process_probe].to_vec(),
            Self::FinishProcessAsReqNeverValid => [process_probe].to_vec(),
            Self::FinishProcessAsReqPolicy => [process_probe].to_vec(),
            Self::FinishProcessAsReqBadoption => [process_probe].to_vec(),
            Self::FinishProcessAsReqEtypeNosupp => [process_probe].to_vec(),
            Self::FinishProcessAsReqSumtypeNosupp => [process_probe].to_vec(),
            Self::FinishProcessAsReqPadataTypeNosupp => [process_probe].to_vec(),
            Self::FinishProcessAsReqTrtypeNosupp => [process_probe].to_vec(),
            Self::FinishProcessAsReqClientRevoked => [process_probe].to_vec(),
            Self::FinishProcessAsReqServiceRevoked => [process_probe].to_vec(),
            Self::FinishProcessAsReqTgtRevoked => [process_probe].to_vec(),
            Self::FinishProcessAsReqClientNotyet => [process_probe].to_vec(),
            Self::FinishProcessAsReqServiceNotyet => [process_probe].to_vec(),
            Self::FinishProcessAsReqKeyExp => [process_probe].to_vec(),
            Self::FinishProcessAsReqPreauthFailed => [process_probe].to_vec(),
            Self::FinishProcessAsReqPreauthRequired => [process_probe].to_vec(),
            Self::FinishProcessAsReqServerNomatch => [process_probe].to_vec(),
            Self::FinishProcessAsReqMustUseUser2user => [process_probe].to_vec(),
            Self::FinishProcessAsReqPathNotAccepted => [process_probe].to_vec(),
            Self::FinishProcessAsReqSvcUnavailable => [process_probe].to_vec(),

            Self::FinishDispatchCacheUnknown => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheNone => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheNameExp => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheServiceExp => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheBadPvno => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheCOldMastKvno => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheSOldMastKvno => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheCPrincipalUnknown => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheSPrincipalUnknown => [dispatch_probe].to_vec(),
            Self::FinishDispatchCachePrincipalNotUnique => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheNullKey => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheCannotPostdate => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheNeverValid => [dispatch_probe].to_vec(),
            Self::FinishDispatchCachePolicy => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheBadoption => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheEtypeNosupp => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheSumtypeNosupp => [dispatch_probe].to_vec(),
            Self::FinishDispatchCachePadataTypeNosupp => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheTrtypeNosupp => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheClientRevoked => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheServiceRevoked => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheTgtRevoked => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheClientNotyet => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheServiceNotyet => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheKeyExp => [dispatch_probe].to_vec(),
            Self::FinishDispatchCachePreauthFailed => [dispatch_probe].to_vec(),
            Self::FinishDispatchCachePreauthRequired => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheServerNomatch => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheMustUseUser2user => [dispatch_probe].to_vec(),
            Self::FinishDispatchCachePathNotAccepted => [dispatch_probe].to_vec(),
            Self::FinishDispatchCacheSvcUnavailable => [dispatch_probe].to_vec(),

            Self::ProcessTgsReqUnknown => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqNone => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqNameExp => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqServiceExp => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqBadPvno => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqCOldMastKvno => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqSOldMastKvno => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqCPrincipalUnknown => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqSPrincipalUnknown => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqPrincipalNotUnique => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqNullKey => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqCannotPostdate => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqNeverValid => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqPolicy => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqBadoption => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqEtypeNosupp => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqSumtypeNosupp => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqPadataTypeNosupp => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqTrtypeNosupp => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqClientRevoked => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqServiceRevoked => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqTgtRevoked => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqClientNotyet => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqServiceNotyet => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqKeyExp => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqPreauthFailed => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqPreauthRequired => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqServerNomatch => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqMustUseUser2user => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqPathNotAccepted => [tgs_req_probe].to_vec(),
            Self::ProcessTgsReqSvcUnavailable => [tgs_req_probe].to_vec(),
        }
    }
}

impl Statistic<AtomicU64, AtomicU32> for Krb5kdcStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        Source::Counter
    }
}

impl TryFrom<&str> for Krb5kdcStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Krb5kdcStatistic::from_str(s)
    }
}
