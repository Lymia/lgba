//! Contains things that integrate with very low level Rust things or the ASM part of the codebase.

mod build_asm;
pub mod gba_header;

// force link the crates.io version of compiler_builtins_local
extern crate compiler_builtins_local;

mod interface {
    use crate::mmio::{
        reg::{BIOS_IRQ_ENTRY, IME},
        sys::Interrupt,
    };

    #[no_mangle]
    pub unsafe extern "C" fn __lgba_init_rust() {
        // initialize IRQs
        BIOS_IRQ_ENTRY.write(entry_interrupt_handler);
        IME.write(true);

        // enable the vblank IRQ
        crate::irq::enable(Interrupt::VBlank);

        // enable the global allocator
        #[cfg(feature = "allocator")]
        crate::sys::allocator::init_rust_alloc();
    }

    #[no_mangle]
    pub unsafe extern "C" fn __lgba_main_func_returned() -> ! {
        crate::panic_handler::static_panic("Internal error: Main function returned?")
    }

    #[instruction_set(arm::a32)]
    pub extern "C" fn entry_interrupt_handler() {
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
    pub static __lgba_exh_lgba_version: &str = env!("CARGO_PKG_VERSION");

    extern "C" {
        pub fn __lgba_abort() -> !;
        pub fn __lgba_TransferBuf(src: *const u8, dst: *mut u8, count: usize);
        pub fn __lgba_ReadByte(src: *const u8) -> u8;
        pub fn __lgba_VerifyBuf(buf1: *const u8, buf2: *const u8, count: usize) -> bool;
    }

    extern "Rust" {
        pub static __lgba_exh_rom_cname: &'static str;
        pub static __lgba_exh_rom_cver: &'static str;
        pub static __lgba_exh_rom_repository: &'static str;

        pub static __lgba_config_canary: u64;
        pub static __lgba_config_int_stack_canary: usize;
        pub static __lgba_config_user_stack_canary: usize;

        pub static __ewram_end: usize;
        pub static __bss_end: usize;
        pub static __lgba_config_iwram_end: usize;
    }
}

#[inline(always)]
pub fn abort() -> ! {
    unsafe {
        interface::__lgba_abort();
    }
}

use core::ops::Range;
pub use interface::{
    __lgba_exh_lgba_version as EXH_LGBA_VERSION, __lgba_exh_rom_cname as EXH_ROM_CNAME,
    __lgba_exh_rom_cver as EXH_ROM_CVER, __lgba_exh_rom_repository as EXH_ROM_REPO,
};

/// Copies data from a given memory address into a buffer.
#[inline(always)]
pub unsafe fn sram_read_raw_buf(dst: &mut [u8], src: usize) {
    if !dst.is_empty() {
        interface::__lgba_TransferBuf(src as _, dst.as_mut_ptr(), dst.len());
    }
}

/// Copies data from a buffer into a given memory address.
#[inline(always)]
pub unsafe fn sram_write_raw_buf(dst: usize, src: &[u8]) {
    if !src.is_empty() {
        interface::__lgba_TransferBuf(src.as_ptr(), dst as _, src.len());
    }
}

/// Verifies that the data in a buffer matches that in a given memory address.
#[inline(always)]
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

pub fn iwram_alloc_range() -> Range<usize> {
    let start = unsafe { &interface::__bss_end as *const _ as usize };
    let end = unsafe { interface::__lgba_config_iwram_end };
    start..end
}
pub fn ewram_alloc_range() -> Range<usize> {
    let start = unsafe { &interface::__ewram_end as *const _ as usize };
    start..0x2040000
}
