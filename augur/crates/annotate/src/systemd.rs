use std::ffi::CStr;

use anyhow::{anyhow, Context};
use augur_common::{Annotator, Sample};
use bstr::{BString, ByteSlice};
use cstr::cstr;
use libc::{c_int, c_void};

type GetUnitFn = unsafe extern "C" fn(libc::pid_t, *mut *mut i8) -> c_int;
type GetSliceFn = unsafe extern "C" fn(libc::pid_t, *mut *mut i8) -> c_int;

pub struct Systemd {
    handle: *mut c_void,
    sd_pid_get_unit: GetUnitFn,
    sd_pid_get_slice: GetSliceFn,
}

impl Systemd {
    pub fn new() -> anyhow::Result<Self> {
        let libpath = cstr!("/lib64/libsystemd.so.0");

        let handle = wrap_dlfunc(|| unsafe { libc::dlopen(libpath.as_ptr(), libc::RTLD_NOW) })
            .context("Unable to load libsystemd.so")?;

        let sd_pid_get_unit = unsafe { loadfn(handle, cstr!("sd_pid_get_unit"))? };
        let sd_pid_get_slice = unsafe { loadfn(handle, cstr!("sd_pid_get_slice"))? };

        Ok(Self {
            handle,
            sd_pid_get_unit,
            sd_pid_get_slice,
        })
    }

    /// Returns the systemd system unit of a process.
    ///
    /// The unit name is a short string, suitable for usage in file system
    /// paths. Note that not all processes are part of a system unit/service
    /// (e.g. user processes or kernel threads). For processes that are not part
    /// of a systemd system unit this function will fail.
    pub fn get_unit(&self, proc: libc::pid_t) -> Option<BString> {
        // sd_pid_get_unit returns the unit for the passed in pid _except_ if
        // that pid is 0 in which case it returns the systemd unit for the
        // calling process. However, when profiling pid 0 actually stands for
        // threads within the kernel idle process which of course don't have a
        // corrsponding systemd unit.
        //
        // If we don't have this if statement then about 70% of the samples get
        // attributed to the augur.service systemd unit which is not exactly
        // useful.
        if proc == 0 {
            return None;
        }

        let mut name = std::ptr::null_mut();
        let res = unsafe { (self.sd_pid_get_unit)(proc, &mut name) };
        if res < 0 || name.is_null() {
            return None;
        }

        // The systemd docs say that name must be freed using libc::free. We can't
        // guarantee that rust is using the same allocator so we copy here and
        // then free the string that systemd returned.
        let _guard = drop_guard::guard(name, |name| unsafe { libc::free(name as _) });

        let cstr = unsafe { CStr::from_ptr(name) };
        Some(BString::from(cstr.to_bytes()))
    }

    /// Get the systemd slice unit that the process is a member of.
    pub fn get_slice(&self, proc: libc::pid_t) -> Option<BString> {
        // proc == 0 means get the info for the current pid. We don't want that.
        if proc == 0 {
            return None;
        }

        let mut name = std::ptr::null_mut();
        let res = unsafe { (self.sd_pid_get_slice)(proc, &mut name) };
        if res < 0 || name.is_null() {
            return None;
        }

        // Need to free using libc::free since rust code may not be using the same
        // allocator.
        let _guard = drop_guard::guard(name, |name| unsafe { libc::free(name as _) });

        let cstr = unsafe { CStr::from_ptr(name) };
        Some(BString::from(cstr.to_bytes()))
    }
}

#[async_trait::async_trait]
impl Annotator for Systemd {
    fn name(&self) -> &str {
        "systemd"
    }

    async fn annotate(&self, sample: &mut Sample) {
        let mut info = sample.systemd.clone().unwrap_or_default();

        info.unit = self.get_unit(sample.pid as _);
        info.slice = self.get_slice(sample.pid as _);

        sample.systemd = Some(info);
    }
}

unsafe impl Send for Systemd {}
unsafe impl Sync for Systemd {}

impl Drop for Systemd {
    fn drop(&mut self) {
        unsafe { libc::dlclose(self.handle) };
    }
}

fn wrap_dlfunc<F: FnOnce() -> *mut c_void>(func: F) -> anyhow::Result<*mut c_void> {
    let result = func();
    if !result.is_null() {
        return Ok(result);
    }

    let error = unsafe { libc::dlerror() };

    // This shouldn't happen unless the wrapped function is doing something wierd.
    if error.is_null() {
        return Err(anyhow!("Unknown error (dlerror returned NULL)"));
    }

    let message = unsafe { CStr::from_ptr(error) };

    Err(anyhow!(message.to_string_lossy()))
}

unsafe fn loadfn<P1, P2, R>(
    handle: *mut c_void,
    name: &CStr,
) -> anyhow::Result<unsafe extern "C" fn(P1, P2) -> R> {
    let fnptr = wrap_dlfunc(|| libc::dlsym(handle, name.as_ptr())).with_context(|| {
        format!(
            "Unable to load {} from libsystemd.so",
            name.to_bytes().to_str_lossy()
        )
    })?;
    Ok(std::mem::transmute(fnptr))
}
