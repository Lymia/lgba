#![no_std]
#![no_main]

#[macro_use]
extern crate lgba;

use core::alloc::{GlobalAlloc, Layout};
use lgba::display::{Terminal, TerminalFontFull};

#[inline(never)]
#[lgba::iwram]
fn main_impl() -> ! {
    let mut terminal = Terminal::new();
    let mut terminal = terminal.activate::<TerminalFontFull>();
    terminal.set_color(1, 0, 0x7F1C);
    terminal.set_color(2, !0, 0);

    for (j, str) in [
        "Hello, world!",
        "",
        "",
        "",
        "",
        "",
        "こんにちは、世界!",
        "",
        "色は匂えど",
        "散りぬるを",
        "我が世誰ぞ",
        "常ならん",
        "有為の奥山",
        "今日越えて",
        "浅き夢見じ",
        "酔いもせず",
    ]
    .iter()
    .enumerate()
    {
        for (i, char) in str.chars().enumerate() {
            terminal.set_char(1 + i, 1 + j, char, 0);
        }
    }
    for (i, char) in "Hello, world! (but it's in half-width characters)"
        .chars()
        .enumerate()
    {
        terminal.set_char_hw(2 + i, 2, char, 0);
    }
    for (i, char) in "Hello, world! (but it's both pink and half-width)"
        .chars()
        .enumerate()
    {
        terminal.set_char_hw(2 + i, 3, char, 1);
    }
    for (i, char) in "Reverse text! Reverse text!"
        .chars()
        .enumerate()
    {
        terminal.set_char(1 + i, 4, char, 2);
    }
    for (i, char) in "Half-width reverse text! Half-width reverse text!"
        .chars()
        .enumerate()
    {
        terminal.set_char_hw(2 + i, 5, char, 2);
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
