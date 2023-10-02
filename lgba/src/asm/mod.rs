//! Contains things that integrate with very low level Rust things or the ASM part of the codebase.

use core::ops::Range;

mod build_asm;

#[cfg(feature = "gba_header")]
pub mod gba_header;

// force link the crates.io version of compiler_builtins_local
extern crate compiler_builtins_local;

mod interface {
    use crate::{
        arm,
        mmio::{
            reg::{BIOS_IRQ_ENTRY, IME},
            sys::Interrupt,
        },
    };
    use core::ops::Range;
    use lgba_common::common::StaticStr;

    #[no_mangle]
    pub unsafe extern "C" fn __lgba_init() {
        // initialize stack canaries
        let int_canary: *mut u64 = __lgba_config_int_stack_canary as *mut u64;
        let user_canary: *mut u64 = __lgba_config_user_stack_canary as *mut u64;
        *int_canary = __lgba_config_canary;
        *user_canary = __lgba_config_canary;

        // initialize the logger
        crate::debug::init_debug();

        // initialize the global allocator
        #[cfg(feature = "allocator")]
        crate::sys::allocator::init_rust_alloc();

        // run ctors
        __lgba_run_ctors();
    }

    #[no_mangle]
    pub unsafe extern "C" fn __lgba_setup() {
        // initialize IRQs
        BIOS_IRQ_ENTRY.write(__lgba_interrupt_handler);
        IME.write(true);

        // enable the vblank IRQ
        crate::irq::enable(Interrupt::VBlank);
    }

    #[no_mangle]
    pub unsafe extern "C" fn __lgba_main_func_returned() -> ! {
        crate::panic_handler::static_panic("Internal error: Main function returned?")
    }

    #[arm]
    #[no_mangle]
    pub unsafe extern "C" fn __lgba_interrupt_handler() {
        crate::irq::interrupt_handler();
    }

    #[no_mangle]
    pub unsafe extern "C" fn __aeabi_idiv0() -> ! {
        crate::panic_handler::static_panic("attempt to divide by 0")
    }

    #[no_mangle]
    pub unsafe extern "C" fn __aeabi_ldiv0() -> ! {
        crate::panic_handler::static_panic("attempt to divide by 0")
    }

    #[no_mangle]
    pub static __lgba_exh_lgba_version: StaticStr = StaticStr::new(env!("CARGO_PKG_VERSION"));

    extern "C" {
        pub fn __lgba_abort() -> !;
        pub fn __lgba_run_ctors();
        pub fn __lgba_TransferBuf(src: *const u8, dst: *mut u8, count: usize);
        pub fn __lgba_ReadByte(src: *const u8) -> u8;
        pub fn __lgba_VerifyBuf(buf1: *const u8, buf2: *const u8, count: usize) -> bool;
    }

    extern "Rust" {
        pub static __lgba_exh_rom_cname: StaticStr;
        pub static __lgba_exh_rom_cver: StaticStr;
        pub static __lgba_exh_rom_repository: StaticStr;

        pub static __lgba_config_canary: u64;
        pub static __lgba_config_int_stack_canary: usize;
        pub static __lgba_config_user_stack_canary: usize;

        pub static __ewram_end: usize;
        pub static __bss_end: usize;
        pub static __lgba_config_iwram_free_end: usize;
    }

    pub fn iwram_free_range() -> Range<usize> {
        let start = unsafe { &__bss_end as *const _ as usize };
        let end = unsafe { __lgba_config_iwram_free_end };
        start..end
    }

    pub fn ewram_free_range() -> Range<usize> {
        let start = unsafe { &__ewram_end as *const _ as usize };
        start..0x2040000
    }

    #[linkage = "weak"]
    #[no_mangle]
    pub fn __lgba_config_alloc_zones(callback: fn(&[Range<usize>])) {
        callback(&[iwram_free_range(), ewram_free_range()])
    }
}

#[inline(always)]
pub fn abort() -> ! {
    unsafe {
        interface::__lgba_abort();
    }
}

pub use interface::{
    __lgba_exh_lgba_version as EXH_LGBA_VERSION, __lgba_exh_rom_cname as EXH_ROM_CNAME,
    __lgba_exh_rom_cver as EXH_ROM_CVER, __lgba_exh_rom_repository as EXH_ROM_REPO,
};

/// Copies data from a given memory address into a buffer.
#[inline(never)]
pub unsafe fn sram_read_raw_buf(dst: &mut [u8], src: usize) {
    if !dst.is_empty() {
        interface::__lgba_TransferBuf(src as _, dst.as_mut_ptr(), dst.len());
    }
}

/// Copies data from a buffer into a given memory address.
#[inline(never)]
pub unsafe fn sram_write_raw_buf(dst: usize, src: &[u8]) {
    if !src.is_empty() {
        interface::__lgba_TransferBuf(src.as_ptr(), dst as _, src.len());
    }
}

/// Verifies that the data in a buffer matches that in a given memory address.
#[inline(never)]
pub unsafe fn sram_verify_raw_buf(buf1: &[u8], buf2: usize) -> bool {
    if !buf1.is_empty() {
        interface::__lgba_VerifyBuf(buf1.as_ptr(), buf2 as _, buf1.len() - 1)
    } else {
        true
    }
}

/// Reads a byte from a given memory address.
#[inline(always)]
pub unsafe fn sram_read_raw_byte(src: usize) -> u8 {
    interface::__lgba_ReadByte(src as _)
}

#[inline(always)]
pub fn check_user_canary() {
    unsafe {
        let offset = interface::__lgba_config_user_stack_canary as *mut u64;
        if *offset != interface::__lgba_config_canary {
            crate::panic_handler::canary_error()
        }
    }
}

#[inline(always)]
pub fn check_interrupt_canary() {
    unsafe {
        let offset = interface::__lgba_config_int_stack_canary as *mut u64;
        if *offset != interface::__lgba_config_canary {
            crate::panic_handler::canary_error()
        }
    }
}

pub fn iwram_free_range() -> Range<usize> {
    interface::iwram_free_range()
}

pub fn ewram_free_range() -> Range<usize> {
    interface::ewram_free_range()
}

pub fn alloc_zones(callback: fn(&[Range<usize>])) {
    interface::__lgba_config_alloc_zones(callback)
}

/// Initializes the internal state of this crate.
///
/// This should be called before any other functions of lgba are called, and should only ever be
/// called once. As it is normally called by the `crt0.s` code provided by lgba, this is only
/// required if you are writing a custom entry point in assembly code.
///
/// This function is available to assembly code under the name `__lgba_init`, even if the
/// `low_level` feature is not enabled.
///
/// # Safety
///
/// This function should only ever be called once, and at the start of your code's execution,
/// before you use any lgba functionality.
#[cfg(feature = "low_level")]
#[doc(cfg(feature = "low_level"))]
pub unsafe fn init_lgba() {
    interface::__lgba_init();
}

/// Sets up the default state of the GBA on startup.
///
/// Specifically, this currently does the following actions:
///
/// * Sets the interrupt handler to `__lgba_interrupt_handler`.
/// * Enables interrupts.
/// * Enables the vblank interrupt in both [`DISPSTAT`] and [`IE`].
///
/// This is separate from [`init_lgba`] because the operations done here are optional and may
/// clash with the hardware configuration needed during tasks such as modding an existing ROM.
/// As with that function, it is called automatically by the `crt0.s` provided by lgba.
///
/// This function is available to assembly code under the name `__lgba_setup`, even if the
/// `low_level` feature is not enabled.
///
/// [`DISPSTAT`]: https://mgba-emu.github.io/gbatek/#4000004h---dispstat---general-lcd-status-readwrite
/// [`IE`]: https://mgba-emu.github.io/gbatek/#4000200h---ie---interrupt-enable-register-rw
///
/// # Safety
///
/// This function is unsafe because it enables interrupts with no checking. See
/// [`set_interrupts_enabled`] for more information.
///
/// [`set_interrupts_enabled`]: crate::irq::set_interrupts_enabled
#[cfg(feature = "low_level")]
#[doc(cfg(feature = "low_level"))]
pub unsafe fn setup_lgba() {
    interface::__lgba_setup();
}

#[cfg(feature = "low_level")]
pub static DEFAULT_INTERRUPT_HANDLER: unsafe extern "C" fn() = interface::__lgba_interrupt_handler;
