#![no_std]
#![no_main]

#[macro_use]
extern crate lgba;

use core::{
    alloc::{GlobalAlloc, Layout},
    fmt::Write,
};
use lgba::{
    display::{Terminal, TerminalFontFull},
    dma::DmaChannelId,
};

#[lgba::entry]
#[rom(title = "LGBA_TERMTST", code = "LGTT")]
fn rom_entry() -> ! {
    let mut terminal = Terminal::new();
    terminal.use_dma_channel(DmaChannelId::Dma3);
    let terminal = terminal.activate::<TerminalFontFull>();
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
    for i in 1..105 {
        if i % 15 == 0 {
            terminal.write_str("[fizzbuzz] ");
        } else if i % 3 == 0 {
            terminal.write_str("[fizz] ");
        } else if i % 5 == 0 {
            terminal.write_str("[buzz] ");
        } else {
            write!(terminal.write(), "{} ", i).unwrap();
        }
    }
    terminal.new_line();

    let mut frame = 0;
    loop {
        terminal.clear_line(18);
        terminal.set_cursor(0, 18);
        write!(terminal.write(), "#{} / {:?}", frame, lgba::sys::pressed_keys())
            .unwrap();
        frame += 1;

        loop {
            let dispstat = unsafe { core::ptr::read_volatile(0x4000004 as *const u16) };
            if dispstat & 1 == 0 {
                break;
            }
        }
        loop {
            let dispstat = unsafe { core::ptr::read_volatile(0x4000004 as *const u16) };
            if dispstat & 1 != 0 {
                break;
            }
        }
    }
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
