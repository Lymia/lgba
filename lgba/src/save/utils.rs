//! A package containing useful utilities for writing save accessors.

use super::Error;
use crate::{
    mmio::{reg::WAITCNT, sys::WaitState},
    sync::{RawMutex, RawMutexGuard},
    timer::{Timer, TimerId, TimerMode},
};

/// A timeout type used to prevent hardware errors in save media from hanging
/// the game.
pub struct Timeout {
    timer: Option<Timer>,
}
impl Timeout {
    /// Creates a new timeout from the timer passed to [`set_timer_for_timeout`].
    ///
    /// ## Errors
    ///
    /// If another timeout has already been created.
    #[inline(never)]
    pub fn new(timer: Option<TimerId>) -> Self {
        Timeout {
            timer: match timer {
                None => None,
                Some(id) => Some(id.create()),
            },
        }
    }

    /// Starts this timeout.
    pub fn start(&mut self) {
        if let Some(timer) = &mut self.timer {
            timer.set_timer_mode(TimerMode::Cycle1024).set_enabled(true);
        }
    }

    /// Returns whether a number of milliseconds has passed since the last call
    /// to [`Timeout::start()`].
    pub fn check_timeout_met(&self, check_ms: u16) -> bool {
        if let Some(timer) = &self.timer {
            check_ms as u32 * 17 < timer.value()
        } else {
            false
        }
    }
}
impl Drop for Timeout {
    fn drop(&mut self) {
        if let Some(timer) = &mut self.timer {
            timer.set_enabled(false);
        }
    }
}

pub fn lock_media_access() -> Result<RawMutexGuard<'static>, Error> {
    static LOCK: RawMutex = RawMutex::new();
    match LOCK.try_lock() {
        Some(x) => Ok(x),
        None => Err(Error::MediaInUse),
    }
}

pub fn set_sram_wait(wait: WaitState) {
    WAITCNT.write(WAITCNT.read().with_sram_wait(wait));
}
