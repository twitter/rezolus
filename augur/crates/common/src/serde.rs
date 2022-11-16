//! Serde helpers for serializing BStrings to a more readable format.
//!
//! By default, BString serializes to a byte array. However, we're using bstring
//! to represent human-readable text that may occasionally have some bytes that
//! are not utf8. We do need to handle the case where the BString contains
//! non-utf8 data but we can get most of the way there by serializing utf8
//! bstrings as strings and everything else as arrays for human-readable
//! formats. BString can deserialize from either so this works transparently
//! other than needing to add some serde annotations otherwise.

use bstr::{BString, ByteSlice};
use serde::{Serialize, Serializer};

pub(crate) fn serialize_bstring<S>(bstring: &BString, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if !ser.is_human_readable() {
        return bstring.serialize(ser);
    }

    match bstring.to_str() {
        Ok(s) => ser.serialize_str(s),
        Err(_) => ser.serialize_bytes(&bstring),
    }
}

pub(crate) fn serialize_opt_bstring<S>(opt: &Option<BString>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    struct BStringWrapper<'a>(&'a BString);
    impl<'a> Serialize for BStringWrapper<'a> {
        fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serialize_bstring(&self.0, ser)
        }
    }

    opt.as_ref().map(BStringWrapper).serialize(ser)
}
