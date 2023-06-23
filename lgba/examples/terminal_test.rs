#![no_std]
#![no_main]

#[macro_use]
extern crate lgba;

use core::{
    alloc::{GlobalAlloc, Layout},
    fmt::Write,
};
use lgba::{
    display::{Terminal},
    dma::DmaChannelId,
};
use lgba::display::TerminalFontBasic;
use lgba::sys::Button;

#[lgba::entry]
#[rom(title = "LGBA_TERMTST", code = "LGTT")]
fn rom_entry() -> ! {
    let mut terminal = Terminal::new();
    terminal.use_dma_channel(DmaChannelId::Dma3);
    let terminal = terminal.activate::<TerminalFontBasic>();
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
    for i in 1..=100 {
        if i % 15 == 0 {
            terminal.set_half_width(true);
            terminal.write_str("[fizz-buzz]");
        } else if i % 3 == 0 {
            terminal.set_half_width(true);
            terminal.write_str("[fizz]");
        } else if i % 5 == 0 {
            terminal.set_half_width(true);
            terminal.write_str("[buzz]");
        } else {
            terminal.set_half_width(false);
            write!(terminal.write(), "{}", i).unwrap();
        }
        terminal.set_half_width(true);
        terminal.write_str(" ");
    }
    terminal.new_line();

    let mut frame = 0;
    loop {
        terminal.clear_line(18);
        terminal.set_cursor(0, 18);
        terminal.set_half_width(false);

        let keys = lgba::sys::pressed_keys();
        write!(terminal.write(), "#{:03} / ", frame).unwrap();
        if keys.is_empty() {
            terminal.set_half_width(true);
            terminal.write_str("(none)");
        } else {
            for key in keys {
                match key {
                    Button::A => terminal.write_str("A"),
                    Button::B => terminal.write_str("B"),
                    Button::Select => terminal.write_str("○"),
                    Button::Start => terminal.write_str("●"),
                    Button::Right => terminal.write_str("→"),
                    Button::Left => terminal.write_str("←"),
                    Button::Up => terminal.write_str("↑"),
                    Button::Down => terminal.write_str("↓"),
                    Button::R => terminal.write_str("R"),
                    Button::L => terminal.write_str("L"),
                }
            }
        }
        frame = (frame + 1) % 1000;

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
