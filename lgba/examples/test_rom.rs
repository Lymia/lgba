#![no_std]
#![no_main]

#[macro_use]
extern crate lgba;

use core::alloc::{GlobalAlloc, Layout};
use lgba::display::{Terminal, TerminalFontFull};

#[inline(never)]
fn main_impl() -> ! {
    let mut terminal = Terminal::new();
    let terminal = terminal.activate::<TerminalFontFull>(true);
    let mut terminal = terminal.lock();

    terminal.set_color(0, lgba::display::rgb_24bpp(54, 131, 255), !0);
    terminal.set_color(1, 0, lgba::display::rgb_24bpp(255, 194, 211));
    terminal.set_color(2, !0, 0);

    terminal.write_str("Hello, world!");
    terminal.new_line();

    terminal.set_half_width(true);
    terminal.write_str("Hello, world! (but it's in half-width characters)");
    terminal.new_line();

    terminal.set_active_color(1);
    terminal.write_str("Hello, world! (but it's both pink and half-width)");
    terminal.new_line();

    terminal.set_half_width(false);
    terminal.set_active_color(2);
    terminal.write_str("Reverse text! Reverse text!");
    terminal.new_line();

    terminal.set_half_width(true);
    terminal.write_str("Half-width reverse text! Half-width reverse text!");
    terminal.new_line();

    terminal.new_line();
    terminal.set_active_color(0);
    terminal.write_str("Word wraptest woraoijoioi aaaaa! Word wraptest woraoijoioi! Word wraptest woraoijoioi! Word wraptest woraoijoioi! Word wraptest woraoijoioi aaaaa! Word wraptest woraoijoioi! Word wraptest woraoijoioi! Word wraptest woraoijoioi! Word wraptest woraoijoioi! Word wraptest woraoijoioi!");

    loop {}
}

#[inline(never)]
fn dbg_long_div(a: u64, b: u64) {
    println!("long div: {} / {} = {}", a, b, a / b);
    println!("long div: {} * {} = {}", a, b, a * b);
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

    dbg_long_div(100000, 1000);
    dbg_long_div(100000, 10000);
    dbg_long_div(100000, 100000);

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
