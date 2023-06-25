use crate::{
    asm::{EXH_LGBA_VERSION, EXH_ROM_CNAME, EXH_ROM_CVER, EXH_ROM_REPO},
    display::{ActiveTerminalAccess, Terminal, TerminalFontAscii},
    eprintln,
    sync::Static,
};
use core::{
    alloc::Layout,
    panic::{Location, PanicInfo},
};

// TODO: Prevent long messages from scrolling off the screen.

#[inline(never)]
fn panic_start() {
    static PANICKING: Static<bool> = Static::new(false);
    // detect double panic
    if PANICKING.replace(true) {
        eprintln!("Panicked while panicking. Aborting!");
        crate::sys::abort();
    }
}

#[inline(never)]
fn write_panic_head(terminal: &mut ActiveTerminalAccess<TerminalFontAscii>) {
    // print a common message for simple UX
    terminal.set_cursor(17, 0);
    terminal.write_str("Fatal Error!\n\n");
    terminal.set_half_width(true);
    terminal.write_str(
        "The Game Pak has encountered a serious error and has shut down to prevent unexpected \
        behavior. Your progress since the last time you saved may have been lost.\n\n",
    );

    // write bug report message
    unsafe {
        if !EXH_ROM_REPO.is_empty() {
            write!(
                terminal.write(),
                "This is likely a bug. You can report it at this URL:\n{}\n\n",
                EXH_ROM_REPO
            );
        }
    }

    // write version - unsafe only because it uses externs
    unsafe {
        write!(
            terminal.write(),
            "Version : {} {} / lgba {}\n",
            EXH_ROM_CNAME,
            EXH_ROM_CVER,
            EXH_LGBA_VERSION,
        );
    }
}
#[inline(never)]
fn write_location(
    terminal: &mut ActiveTerminalAccess<TerminalFontAscii>,
    location: Option<&Location>,
) {
    match location {
        None => terminal.write_str("Location: <unknown>\n"),
        Some(location) => write!(terminal.write(), "Location: {}\n", location),
    }
}

fn panic_with_term(func: impl FnOnce(&mut ActiveTerminalAccess<TerminalFontAscii>)) -> ! {
    // set up the graphical terminal with a basic font
    let mut terminal = Terminal::new();
    terminal.set_color(0, crate::display::rgb_24bpp(200, 0, 0), !0);
    let terminal = terminal.activate_no_lock::<TerminalFontAscii>();
    let mut terminal = terminal.lock();

    // run the actual function
    func(&mut terminal);

    // abort cleanly
    crate::sys::abort()
}

#[inline(never)]
fn handle_static_panic(message: &str, location: Option<&Location>) -> ! {
    crate::irq::disable(|| crate::dma::pause_dma(|| handle_static_panic_inner(message, location)))
}
fn handle_static_panic_inner(message: &str, location: Option<&Location>) -> ! {
    panic_start();

    // print the panic message to debug terminal, if we have one
    match location {
        Some(location) => eprintln!("ROM panicked: {} @ {}", message, location),
        None => eprintln!("ROM panicked: {}", message),
    }

    // show a panic screen
    panic_with_term(|terminal| {
        write_panic_head(terminal);
        write_location(terminal, location);
        write!(terminal.write(), "Message : {}\n", message);
    })
}

#[panic_handler]
#[inline(never)]
fn handle_panic(error: &PanicInfo) -> ! {
    crate::irq::disable(|| crate::dma::pause_dma(|| handle_panic_inner(error)))
}
fn handle_panic_inner(error: &PanicInfo) -> ! {
    panic_start();

    // print the panic message to debug terminal, if we have one
    eprintln!("ROM panicked: {}", error);

    // show a panic screen
    panic_with_term(|terminal| {
        write_panic_head(terminal);
        write_location(terminal, error.location());

        // write panic message
        match error.message() {
            None => terminal.write_str("Message : <unknown>\n"),
            Some(error) => write!(terminal.write(), "Message : {}\n", error),
        }
    })
}

#[inline(never)]
fn handle_alloc_panic(layout: Layout) -> ! {
    crate::irq::disable(|| crate::dma::pause_dma(|| handle_alloc_panic_inner(layout)))
}
fn handle_alloc_panic_inner(layout: Layout) -> ! {
    panic_start();

    // print the panic message to debug terminal, if we have one
    eprintln!("out of memory: {:?}", layout);

    // show a panic screen
    panic_with_term(|terminal| {
        write_panic_head(terminal);
        write_location(terminal, None);
        write!(terminal.write(), "Message : Ran out of memory.\nLayout  :{:?}\n", layout);
    })
}

#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    handle_alloc_panic(layout)
}

#[track_caller]
pub fn static_panic(msg: &str) -> ! {
    handle_static_panic(msg, Some(Location::caller()))
}
