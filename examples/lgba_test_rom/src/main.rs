#![no_std]
#![no_main]

#[macro_use]
extern crate lgba;
extern crate alloc;

mod game_data_test;
mod interrupt_test;
mod savegame_test;
mod terminal_test;

use core::pin::pin;
use lgba::{
    display::{Terminal, TerminalFontBasic},
    dma::DmaChannelId,
    irq::{Interrupt, InterruptHandler},
    sys::Button,
};

static OPTIONS: &[(&'static str, fn() -> !)] = &[
    ("Test terminal function", || terminal_test::run()),
    ("Test savegame function", || savegame_test::run()),
    ("Test interrupt handlers", || interrupt_test::run()),
    ("Test game data", || game_data_test::run()),
    ("Test panic handler", || {
        panic!("oh no something really bad happened!!! help!!!")
    }),
];

#[lgba::ctor]
fn ctor_test_1() {
    lgba::println!("Hello world from #[ctor]! (1)");
}

#[lgba::ctor]
fn ctor_test_2() {
    lgba::println!("Hello world from #[ctor]! (2a)");
}

#[lgba::entry]
#[rom(title = "LGBA_TESTROM", code = "LGTR")]
fn rom_entry() -> ! {
    let mut terminal = Terminal::new();
    terminal.use_dma_channel(DmaChannelId::Dma3);
    terminal.set_force_blank(true);
    let active_terminal = terminal.activate::<TerminalFontBasic>();
    let mut terminal = active_terminal.lock();

    terminal.write_str(concat!("lgba test rom v", env!("CARGO_PKG_VERSION"), "\n"));
    terminal.write_str("-----------------------------\n");

    terminal.set_cursor(0, 18);
    terminal.write_str("Press ↓○A to reset any test");

    for (i, (name, _)) in OPTIONS.iter().enumerate() {
        terminal.set_cursor(3, i + 3);
        terminal.write_str(name);
    }

    lgba::sys::wait_for_vblank();
    terminal.set_force_blank(false);

    lgba::sys::set_keypad_irq_combo(Button::Down | Button::Select | Button::A);
    let handler = pin!(InterruptHandler::new(|| lgba::sys::reset()));
    handler.register(Interrupt::Keypad);
    lgba::irq::enable(Interrupt::Keypad);

    let mut cursor_pos = 0;
    let mut last_held = lgba::sys::pressed_keys();
    loop {
        lgba::sys::wait_for_vblank();

        let held = lgba::sys::pressed_keys();
        let pressed = held - last_held;
        last_held = held;

        terminal.set_char_full(0, cursor_pos + 3, ' ', 0);
        if pressed.contains(Button::Up) {
            if cursor_pos == 0 {
                cursor_pos = OPTIONS.len() - 1;
            } else {
                cursor_pos -= 1;
            }
        } else if pressed.contains(Button::Down) {
            cursor_pos += 1;
            if cursor_pos == OPTIONS.len() {
                cursor_pos = 0;
            }
        }
        terminal.set_char_full(0, cursor_pos + 3, '>', 0);

        if pressed.contains(Button::A) {
            drop(terminal);
            drop(active_terminal);
            OPTIONS[cursor_pos].1();
        }
    }
}
