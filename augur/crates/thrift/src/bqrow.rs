// Autogenerated by Thrift Compiler (0.16.0)
// DO NOT EDIT UNLESS YOU ARE SURE THAT YOU KNOW WHAT YOU ARE DOING

#![allow(unused_imports)]
#![allow(unused_extern_crates)]
#![allow(clippy::too_many_arguments, clippy::type_complexity, clippy::vec_box)]
#![cfg_attr(rustfmt, rustfmt_skip)]

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::{From, TryFrom};
use std::default::Default;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use thrift::OrderedFloat;
use thrift::{ApplicationError, ApplicationErrorKind, ProtocolError, ProtocolErrorKind, TThriftClient};
use thrift::protocol::{TFieldIdentifier, TListIdentifier, TMapIdentifier, TMessageIdentifier, TMessageType, TInputProtocol, TOutputProtocol, TSetIdentifier, TStructIdentifier, TType};
use thrift::protocol::field_id;
use thrift::protocol::verify_expected_message_type;
use thrift::protocol::verify_expected_sequence_number;
use thrift::protocol::verify_expected_service_call;
use thrift::protocol::verify_required_field_exists;
use thrift::server::TProcessor;

//
// BQSample
//

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BQSample {
  pub timestamp: String,
  pub pid: Option<i32>,
  pub tid: Option<i32>,
  pub cpu: Option<i32>,
  pub hostname: Option<String>,
  pub source: Option<String>,
  pub command: Option<String>,
  pub callstack: Option<Vec<String>>,
  pub job: Option<String>,
  pub instance_id: Option<i32>,
  pub thread_name: Option<String>,
}

impl BQSample {
  pub fn new<F2, F3, F4, F5, F6, F7, F8, F9, F10, F11>(timestamp: String, pid: F2, tid: F3, cpu: F4, hostname: F5, source: F6, command: F7, callstack: F8, job: F9, instance_id: F10, thread_name: F11) -> BQSample where F2: Into<Option<i32>>, F3: Into<Option<i32>>, F4: Into<Option<i32>>, F5: Into<Option<String>>, F6: Into<Option<String>>, F7: Into<Option<String>>, F8: Into<Option<Vec<String>>>, F9: Into<Option<String>>, F10: Into<Option<i32>>, F11: Into<Option<String>> {
    BQSample {
      timestamp,
      pid: pid.into(),
      tid: tid.into(),
      cpu: cpu.into(),
      hostname: hostname.into(),
      source: source.into(),
      command: command.into(),
      callstack: callstack.into(),
      job: job.into(),
      instance_id: instance_id.into(),
      thread_name: thread_name.into(),
    }
  }
  pub fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<BQSample> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = None;
    let mut f_2: Option<i32> = None;
    let mut f_3: Option<i32> = None;
    let mut f_4: Option<i32> = None;
    let mut f_5: Option<String> = None;
    let mut f_6: Option<String> = None;
    let mut f_7: Option<String> = None;
    let mut f_8: Option<Vec<String>> = None;
    let mut f_9: Option<String> = None;
    let mut f_10: Option<i32> = None;
    let mut f_11: Option<String> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_string()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_i32()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_i32()?;
          f_3 = Some(val);
        },
        4 => {
          let val = i_prot.read_i32()?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_string()?;
          f_5 = Some(val);
        },
        6 => {
          let val = i_prot.read_string()?;
          f_6 = Some(val);
        },
        7 => {
          let val = i_prot.read_string()?;
          f_7 = Some(val);
        },
        8 => {
          let list_ident = i_prot.read_list_begin()?;
          let mut val: Vec<String> = Vec::with_capacity(list_ident.size as usize);
          for _ in 0..list_ident.size {
            let list_elem_0 = i_prot.read_string()?;
            val.push(list_elem_0);
          }
          i_prot.read_list_end()?;
          f_8 = Some(val);
        },
        9 => {
          let val = i_prot.read_string()?;
          f_9 = Some(val);
        },
        10 => {
          let val = i_prot.read_i32()?;
          f_10 = Some(val);
        },
        11 => {
          let val = i_prot.read_string()?;
          f_11 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("BQSample.timestamp", &f_1)?;
    let ret = BQSample {
      timestamp: f_1.expect("auto-generated code should have checked for presence of required fields"),
      pid: f_2,
      tid: f_3,
      cpu: f_4,
      hostname: f_5,
      source: f_6,
      command: f_7,
      callstack: f_8,
      job: f_9,
      instance_id: f_10,
      thread_name: f_11,
    };
    Ok(ret)
  }
  pub fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("BQSample");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("timestamp", TType::String, 1))?;
    o_prot.write_string(&self.timestamp)?;
    o_prot.write_field_end()?;
    if let Some(fld_var) = self.pid {
      o_prot.write_field_begin(&TFieldIdentifier::new("pid", TType::I32, 2))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.tid {
      o_prot.write_field_begin(&TFieldIdentifier::new("tid", TType::I32, 3))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.cpu {
      o_prot.write_field_begin(&TFieldIdentifier::new("cpu", TType::I32, 4))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.hostname {
      o_prot.write_field_begin(&TFieldIdentifier::new("hostname", TType::String, 5))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.source {
      o_prot.write_field_begin(&TFieldIdentifier::new("source", TType::String, 6))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.command {
      o_prot.write_field_begin(&TFieldIdentifier::new("command", TType::String, 7))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.callstack {
      o_prot.write_field_begin(&TFieldIdentifier::new("callstack", TType::List, 8))?;
      o_prot.write_list_begin(&TListIdentifier::new(TType::String, fld_var.len() as i32))?;
      for e in fld_var {
        o_prot.write_string(e)?;
      }
      o_prot.write_list_end()?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.job {
      o_prot.write_field_begin(&TFieldIdentifier::new("job", TType::String, 9))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.instance_id {
      o_prot.write_field_begin(&TFieldIdentifier::new("instance_id", TType::I32, 10))?;
      o_prot.write_i32(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.thread_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("thread_name", TType::String, 11))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

