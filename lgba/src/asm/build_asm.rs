use core::arch::global_asm;

macro_rules! include_global_asm {
    ($file:expr) => {
        global_asm!(concat!(include_str!($file), "\n.pool\n.text\n.thumb\n",), options(raw),);
    };
}

// lgba functions implemented in assembly
include_global_asm!("impl/crt0.s");
include_global_asm!("impl/header.s");
include_global_asm!("impl/save.s");
include_global_asm!("impl/sys.s");

#[cfg(feature = "gba_header")]
include_global_asm!("impl/gba_header.s");

// aeabi functions implemented in assembly
// original source: https://github.com/bobbl/libaeabi-cortexm0
include_global_asm!("aeabi/idiv.S");
include_global_asm!("aeabi/idivmod.S");
include_global_asm!("aeabi/lasr.S");
include_global_asm!("aeabi/ldivmod.S");
include_global_asm!("aeabi/llsl.S");
include_global_asm!("aeabi/llsr.S");
include_global_asm!("aeabi/lmul.S");
include_global_asm!("aeabi/memmove.S");
include_global_asm!("aeabi/memset.S");
