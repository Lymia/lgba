//! A module allowing access to the GBA's timer hardware.
//!
//! For further information, see the [GBATEK documentation] on timers.
//!
//! [GBATEK documentation]: https://mgba-emu.github.io/gbatek/#gbatimers

use crate::{
    mmio::{
        reg::{TM_CNT_H, TM_CNT_L},
        sys::{TimerCnt, TimerScale},
    },
    sync::{RawMutex, RawMutexGuard},
};

static TIMER_LOCK: [RawMutex; 4] =
    [RawMutex::new(), RawMutex::new(), RawMutex::new(), RawMutex::new()];

/// Used to specify a particular timer ID.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
#[repr(u8)]
pub enum TimerId {
    /// The first timer.
    ///
    /// Cascades from nothing and cascades to `Timer2`.
    Timer0,
    /// The second timer.
    ///
    /// Cascades from `Timer1` and cascades to `Timer3`.
    Timer1,
    /// The third timer.
    ///
    /// Cascades from `Timer2` and cascades to `Timer4`.
    Timer2,
    /// The fourth timer.
    ///
    /// Cascades from `Timer3` and cascades to nothing.
    Timer3,
}
impl TimerId {
    /// Returns the timer that this one cascades to
    pub fn cascade_destination(self) -> Option<TimerId> {
        match self {
            TimerId::Timer0 => Some(TimerId::Timer1),
            TimerId::Timer1 => Some(TimerId::Timer2),
            TimerId::Timer2 => Some(TimerId::Timer3),
            TimerId::Timer3 => None,
        }
    }

    /// Returns the timer that this one cascades from
    pub fn cascade_source(self) -> Option<TimerId> {
        match self {
            TimerId::Timer0 => None,
            TimerId::Timer1 => Some(TimerId::Timer0),
            TimerId::Timer2 => Some(TimerId::Timer1),
            TimerId::Timer3 => Some(TimerId::Timer2),
        }
    }

    /// Creates a new timer from this one.
    pub fn create(self) -> Timer {
        Timer {
            id: self,
            cnt: TimerCnt::default(),
            reset_to: 0,
            _lock: TIMER_LOCK[self as usize].lock(),
        }
    }
}

/// Controls the way the timer is incremented.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum TimerMode {
    /// The timer will increase once for every clock cycle the GBA executes.
    ///
    /// This happens exactly 2<sup>24</sup> (about 16.7 million) times per second, and exactly
    /// 280896 times per frame.
    ///
    /// Note that these numbers can be used to derive all other durations/frequencies found in the
    /// documentation.
    ///
    /// A duration of one microsecond lasts approximately 16.8 timer cycles, and a duration of
    /// one millisecond lasts approximately 16777 timer cycles.
    Cycle1,
    /// The timer will increase once for every 64 clock cycles the GBA executes.
    ///
    /// This happens approximately 1 million times per second, and exactly 4389 times per frame.
    ///
    /// A duration of one millisecond lasts approximately 262 timer cycles.
    Cycle64,
    /// The timer will increase once for every 256 clock cycles the GBA executes.
    ///
    /// This happens exactly 65536 times per second, and exactly 1097.25 times per frame.
    ///
    /// A duration of one millisecond lasts approximately 65.5 timer cycles.
    Cycle256,
    /// The timer will increase once for every 1024 clock cycles the GBA executes.
    ///
    /// This happens exactly 16384 times per second, and approximately 274.3 times per frame.
    ///
    /// A duration of one millisecond lasts approximately 16.4 timer cycles.
    Cycle1024,
    /// The timer will increase every time the previous timer overflows.
    ///
    /// This mode can only be used with timers 1, 2 or 3, as there is no timer before timer 0.
    Cascade,
}

/// Used to control a GBA timer.
///
/// When this object is dropped, the timer is automatically disabled.
pub struct Timer {
    id: TimerId,
    cnt: TimerCnt,
    reset_to: u16,
    _lock: RawMutexGuard<'static>,
}
impl Timer {
    /// Sets the timer mode in use.
    ///
    /// By default, the timer increments once per processor cycle.
    #[track_caller]
    pub fn set_timer_mode(&mut self, mode: TimerMode) -> &mut Self {
        if self.id == TimerId::Timer0 && mode == TimerMode::Cascade {
            timer_cannot_cascade()
        }
        self.cnt = match mode {
            TimerMode::Cycle1 => self.cnt.with_scale(TimerScale::NoDiv).with_cascade(false),
            TimerMode::Cycle64 => self.cnt.with_scale(TimerScale::Div64).with_cascade(false),
            TimerMode::Cycle256 => self.cnt.with_scale(TimerScale::Div256).with_cascade(false),
            TimerMode::Cycle1024 => self.cnt.with_scale(TimerScale::Div1024).with_cascade(false),
            TimerMode::Cascade => self.cnt.with_cascade(true),
        };
        if self.cnt.enabled() {
            TM_CNT_H.index(self.id as usize).write(self.cnt);
        }
        self
    }

    /// Sets whether this timer should send an interrupt when it overflows.
    ///
    /// By default, no interrupt is sent.
    pub fn set_interrupt_enabled(&mut self, enabled: bool) -> &mut Self {
        self.cnt = self.cnt.with_enable_irq(enabled);
        if self.cnt.enabled() {
            TM_CNT_H.index(self.id as usize).write(self.cnt);
        }
        self
    }

    /// Sets whether the timer is enabled.
    ///
    /// If this causes the state of the timer to change from disabled to enabled, the timer is
    /// automatically reset by hardware without triggering an interrupt.
    ///
    /// By default, it is disabled.
    pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
        self.cnt = self.cnt.with_enabled(enabled);
        if enabled {
            TM_CNT_L.index(self.id as usize).write(self.reset_to);
        }
        TM_CNT_H.index(self.id as usize).write(self.cnt);
        self
    }

    /// Sets the threshold at which the timer resets.
    ///
    /// When the timer reaches this value, it is immediately reset back to 0. If the timer
    /// interrupt is enabled, an interrupt is also sent at that time.
    ///
    /// The overflow value must be between 1 and 65536 inclusive. If this function is called when
    /// the timer is already active, [`value`](`Timer::value`) will return incorrect values until
    /// the next time the timer resets.
    ///
    /// This is a higher-level API over [`set_reset_to`](`Timer::set_reset_to`) and
    /// [`raw_value`](`Timer::raw_value`). If you need to change the reset value while the timer is
    #[track_caller]
    pub fn set_overflow_at(&mut self, value: u32) -> &mut Self {
        if value == 0 || value > 65536 {
            timer_cascade_range();
        }
        self.set_reset_to((65536 - value) as u16)
    }

    /// Returns the value of the timer.
    ///
    /// If the timer is disabled, this is always zero. Otherwise, it is the number of cycles that
    /// have passed since the last time the timer has been reset.
    ///
    /// If [`set_reset_to`](`Timer::set_reset_to`) or [`set_overflow_at`](`Timer::set_overflow_at`)
    /// have been called since the last time the timer reset, this function will return
    /// inconsistant values.
    ///
    /// If you need to do so in your code, use [`set_reset_to`](`Timer::set_reset_to`) and
    /// [`raw_value`](`Timer::raw_value`) rather than this function.
    pub fn value(&self) -> u32 {
        (TM_CNT_L
            .index(self.id as usize)
            .read()
            .wrapping_sub(self.reset_to)) as u32
    }

    /// Resets the timer immediately without triggering an interrupt.
    pub fn reset(&mut self) {
        if self.cnt.enabled() {
            self.set_enabled(false);
            self.set_enabled(true);
        }
    }

    /// Sets the raw value the timer resets to when it overflows.
    ///
    /// This allows more direct control of the timer (with a less convinent API) than
    /// [`set_overflow_at`](`Timer::set_overflow_at`). For most purposes, you should use that
    /// function and [`value`](`Timer::value`) rather than this function and
    /// [`raw_value`](`Timer::raw_value`).
    ///
    /// Whenever the raw timer value overflows (when it reaches 65536) or otherwise is reset, the
    /// timer is set to this value. When this happens due to an overflow and the timer interrupt is
    /// enabled, an interrupt is also sent.
    ///
    /// If this function is called when the timer is already active, [`value`](`Timer::value`) will
    /// return incorrect values until the next time the timer resets.
    #[track_caller]
    pub fn set_reset_to(&mut self, value: u16) -> &mut Self {
        self.reset_to = value;
        if self.cnt.enabled() {
            TM_CNT_L.index(self.id as usize).write(self.reset_to);
        }
        self
    }

    /// Returns the raw value of the timer.
    ///
    /// This corresponds directly to the timer value on the underlying hardware.
    ///
    /// See [`set_reset_to`](`Timer::set_reset_to`) for further information.
    pub fn raw_value(&self) -> u16 {
        TM_CNT_L.index(self.id as usize).read()
    }
}
impl Drop for Timer {
    fn drop(&mut self) {
        if self.cnt.enabled() {
            TM_CNT_H.index(self.id as usize).write(TimerCnt::default());
        }
    }
}

#[inline(never)]
#[track_caller]
fn timer_cannot_cascade() -> ! {
    crate::panic_handler::static_panic("Timer 0 cannot be cascaded!")
}

#[inline(never)]
#[track_caller]
fn timer_cascade_range() -> ! {
    crate::panic_handler::static_panic("Reset threshold must be between 1 and 65536 inclusive.")
}

/// Returns the number of cycles a function takes to execute.
///
/// This function uses timers 2 and 3 and will panic if either is already in use.
pub fn time_cycles(func: impl FnOnce()) -> u32 {
    let mut timer0 = TimerId::Timer2.create();
    let mut timer1 = TimerId::Timer3.create();
    timer0.set_enabled(true);
    timer1.set_timer_mode(TimerMode::Cascade).set_enabled(true);

    func();

    let time = timer0.value() | (timer1.value() << 16);
    time
}
