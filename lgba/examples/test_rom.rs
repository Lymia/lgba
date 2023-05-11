#![no_std]
#![no_main]

#[macro_use]
extern crate lgba;

use core::alloc::{GlobalAlloc, Layout};
use lgba::display::{Terminal, TerminalFontBasic};

#[inline(never)]
#[lgba::iwram]
fn main_impl() -> ! {
    let mut terminal = Terminal::new();
    let terminal = terminal.activate::<TerminalFontBasic>();

    for (i, char) in "Hello, world!".chars().enumerate() {
        println!("{} {}", i, char);
        terminal.set_char(1 + i, 1, char, 0);
    }

    loop {}
}

#[lgba::entry]
#[rom(title = "LGBA_TESTROM", code = "LGTR")]
fn rom_entry() -> ! {
    log!(Info, "log~");
    println!("println~");
    eprintln!("eprintln~");
    error!("error~");
    warn!("warn~");
    info!("info~");
    debug!("debug~");
    dbg!();
    dbg!(dbg!(3) + 3);
    main_impl()
}

struct NoAlloc;
unsafe impl GlobalAlloc for NoAlloc {
    unsafe fn alloc(&self, _: Layout) -> *mut u8 {
        todo!()
    }
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        todo!()
    }
}
#[global_allocator]
static ALLOC: NoAlloc = NoAlloc;
