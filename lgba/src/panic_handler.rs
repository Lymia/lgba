use crate::{
    debug::DebugLevel,
    display::{Terminal, TerminalFontAscii},
    eprintln,
    sync::Static,
};
use core::{alloc::Layout, fmt::Write, panic::PanicInfo};

extern "Rust" {
    pub static __lgba_exh_lgba_version: &'static str;
    pub static __lgba_exh_rom_cname: &'static str;
    pub static __lgba_exh_rom_cver: &'static str;
    pub static __lgba_exh_rom_repository: &'static str;
}

static PANICKING: Static<bool> = Static::new(false);

#[panic_handler]
fn handle_panic(error: &PanicInfo) -> ! {
    // TODO: Prevent long messages from scrolling off the screen.

    // detect double panic
    if PANICKING.replace(true) {
        eprintln!("Panicked while panicking. Aborting!");
        crate::sys::abort();
    }

    // print the panic message to debug terminal, if we have one
    eprintln!("ROM panicked: {}", error);

    // set up the graphical terminal with a basic font
    let mut terminal = Terminal::new();
    terminal.set_color(0, crate::display::rgb_24bpp(150, 0, 0), !0);
    let terminal = terminal.activate_no_lock::<TerminalFontAscii>(false);
    let mut terminal = terminal.lock();

    // print a common message for simple UX
    terminal.write_str("Fatal Error!\n\n");
    terminal.set_half_width(true);
    terminal.write_str(
        "The Game Pak has encountered a serious error and has shut down to prevent unexpected \
        behavior. Your progress since the last time you saved may have been lost.\n\n",
    );

    // write bug report message
    unsafe {
        if !__lgba_exh_rom_repository.is_empty() {
            write!(
                terminal.write(),
                "This is likely a bug. You can report it at this URL:\n{}\n\n",
                __lgba_exh_rom_repository
            )
            .unwrap();
        }
    }

    // write version - unsafe only because it uses externs
    unsafe {
        write!(
            terminal.write(),
            "Version: {} {} / lgba {}\n",
            __lgba_exh_rom_cname,
            __lgba_exh_rom_cver,
            __lgba_exh_lgba_version,
        )
        .unwrap();
    }

    // write panic location
    match error.location() {
        None => terminal.write_str("Location: <unknown>\n"),
        Some(location) => write!(terminal.write(), "Location: {}\n", location).unwrap(),
    }

    // write panic message
    match error.message() {
        None => terminal.write_str("Message: <unknown>\n"),
        Some(error) => write!(terminal.write(), "Message: {}\n", error).unwrap(),
    }

    crate::sys::abort()
}

#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    eprintln!("Could not allocate memory: {:?}", layout);
    crate::sys::abort()
}

pub fn static_panic(msg: &str) -> ! {
    crate::debug::debug_print(DebugLevel::Error, msg);
    crate::sys::abort()
}
