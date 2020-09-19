// The Windows API doesn't call these 'futexes', but the api of WaitOnAddress
// and WakeByAddress is the same as the basic futex api on other systems.

use crate::sync::atomic::AtomicI32;
use crate::sys::{c, dur2timeout};
use crate::time::Duration;

pub fn futex_wait(futex: &AtomicI32, expected: i32, timeout: Option<Duration>) {
    unsafe {
        c::WaitOnAddress(
            futex as *const AtomicI32 as _,
            &expected as *const i32 as _,
            4,
            timeout.map_or(c::INFINITE, dur2timeout),
        );
    }
}

pub fn futex_wake(futex: &AtomicI32) {
    unsafe {
        c::WakeByAddressSingle(futex as *const AtomicI32 as _);
    }
}
