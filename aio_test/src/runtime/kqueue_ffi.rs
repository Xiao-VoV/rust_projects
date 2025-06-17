#![allow(non_camel_case_types, non_snake_case)]
use libc::{c_int, c_void, intptr_t, timespec, uintptr_t};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct kevent {
    pub ident: uintptr_t,
    pub filter: i16,
    pub flags: u16,
    pub fflags: u32,
    pub data: intptr_t,
    pub udata: *mut c_void, //用户数据
}

pub const EVFILT_READ: i16 = -1;
pub const EV_ADD: u16 = 0x1;
pub const EV_ENABLE: u16 = 0x4;
pub const EV_CLEAR: u16 = 0x20;
#[allow(unused)]
pub const EV_DELETE: u16 = 0x2;

unsafe extern "C" {
    pub unsafe fn kqueue() -> c_int;

    pub unsafe fn kevent(
        kq: c_int,
        changelist: *const kevent,
        nchanges: c_int,
        eventlist: *mut kevent,
        nevents: c_int,
        timeout: *const timespec,
    ) -> c_int;

    pub unsafe fn close(fd: c_int) -> c_int;
}

pub struct Event(pub kevent);

impl Event {
    pub fn token(&self) -> usize {
        self.0.udata as usize
    }
}
