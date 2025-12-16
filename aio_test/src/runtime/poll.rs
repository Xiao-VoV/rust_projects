use crate::runtime::kqueue_ffi as ffi;
use libc::{EV_ADD, uintptr_t};
use std::io::{self, Result};
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::time::Duration;

pub type Event = Vec<ffi::Event>;

#[derive(Debug)]
pub struct Registery {
    kq: i32,
}

impl Registery {
    pub fn register(&self, source: &TcpStream, token: usize, interests: i16) -> Result<()> {
        let flags = ffi::EV_ADD | ffi::EV_ENABLE | ffi::EV_CLEAR;

        let kev = ffi::kevent {
            ident: source.as_raw_fd() as uintptr_t,
            filter: interests,
            flags,
            fflags: 0,
            data: 0,
            udata: token as *mut libc::c_void,
        };

        let changelist = [kev];
        let nchange = 1;
        let eventlist = ptr::null_mut();
        let nenents = 0;
        let timeout = ptr::null();

        let res = unsafe {
            ffi::kevent(
                self.kq,
                changelist.as_ptr(),
                nchange,
                eventlist,
                nenents,
                timeout,
            )
        };

        if res == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn deregister(&self, source: &TcpStream) -> Result<()> {
        let kev = ffi::kevent {
            ident: source.as_raw_fd() as uintptr_t,
            filter: ffi::EVFILT_READ,
            flags: ffi::EV_DELETE,
            fflags: 0,
            data: 0,
            udata: ptr::null_mut(),
        };
        let changelist = [kev];

        let res = unsafe {
            ffi::kevent(
                self.kq,
                changelist.as_ptr(),
                1,
                ptr::null_mut(),
                0,
                ptr::null(),
            )
        };
        if res == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

pub struct Poll {
    registry: Registery,
}

impl Poll {
    pub fn new() -> Result<Self> {
        let kq = unsafe { ffi::kqueue() };
        if kq == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(Poll {
            registry: Registery { kq },
        })
    }

    pub fn register(&self) -> &Registery {
        &self.registry
    }

    pub fn poll(&self, timeout: Option<Duration>) -> Result<Event> {}
}
