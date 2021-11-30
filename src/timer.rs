// Copyright 2019 TiKV Project Authors. Licensed under Apache-2.0.

use std::os::raw::c_int;
use std::ptr::null_mut;

use libc;

#[repr(C)]
#[derive(Clone)]
struct Timeval {
    pub tv_sec: i64,
    pub tv_usec: i64,
}

#[repr(C)]
#[derive(Clone)]
struct Itimerval {
    pub it_interval: Timeval,
    pub it_value: Timeval,
}

pub struct Timer {
    _frequency: c_int,
    timerid: libc::timer_t,
}

impl Timer {
    pub fn new(frequency: c_int) -> Timer {
        let interval = 1e9 as i64 / i64::from(frequency);

        let it_interval = libc::timespec {
                tv_sec: interval / 1e9 as i64,
                tv_nsec: interval % 1e9 as i64,
        };
        eprintln!("timer interval: {} {}", interval / 1e9 as i64, interval % 1e9 as i64);

        let timerspec = libc::itimerspec {
            it_interval,
            it_value: it_interval,
        };

        let this_tid = unsafe { libc::gettid() };

        let mut sev: libc::sigevent = unsafe { std::mem::zeroed() };
        sev.sigev_signo = libc::SIGALRM;
        sev.sigev_notify = libc::SIGEV_THREAD_ID;
        sev.sigev_notify_thread_id = this_tid;
        let mut timerid: libc::timer_t = unsafe { std::mem::zeroed() };

        let result = 
            unsafe {
                libc::timer_create(libc::CLOCK_REALTIME,
                                   &mut sev,
                                   &mut timerid)
            };
        eprintln!("timer_create for thread {}: {}", this_tid, result);

        let result =
            unsafe {
                libc::timer_settime(timerid, 0, &timerspec, null_mut())
            };
        eprintln!("timer_set for thread {}: {}", this_tid, result);

        Timer {
            _frequency: frequency,
            timerid,
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        unsafe {
            libc::timer_delete(self.timerid);
        }
    }
}
