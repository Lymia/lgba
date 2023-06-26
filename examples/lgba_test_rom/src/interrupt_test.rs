use core::pin::pin;
use enumset::EnumSet;
use lgba::{
    display::{Terminal, TerminalFontBasic},
    dma::DmaChannelId,
    irq::{Interrupt, InterruptHandler},
    sync::Static,
    timer::{TimerId, TimerMode},
};

pub fn run() -> ! {
    let mut terminal = Terminal::new();
    terminal.use_dma_channel(DmaChannelId::Dma3);
    let terminal = terminal.activate::<TerminalFontBasic>();
    let mut terminal = terminal.lock();

    const STATIC_INIT: Static<u64> = Static::new(0);
    let interrupt_count = [STATIC_INIT; 14];
    macro_rules! handler {
        ($interrupt:expr) => {
            let handler = pin!(InterruptHandler::new(|| {
                let new_ct = interrupt_count[$interrupt as usize].read() + 1;
                interrupt_count[$interrupt as usize].write(new_ct);
            }));
            handler.register($interrupt);
        };
    }
    handler!(Interrupt::VBlank);
    handler!(Interrupt::HBlank);
    handler!(Interrupt::VCounter);
    handler!(Interrupt::Timer0);
    handler!(Interrupt::Timer1);
    handler!(Interrupt::Timer2);
    handler!(Interrupt::Timer3);
    handler!(Interrupt::Serial);
    handler!(Interrupt::Dma0);
    handler!(Interrupt::Dma1);
    handler!(Interrupt::Dma2);
    handler!(Interrupt::Dma3);
    handler!(Interrupt::Keypad);
    handler!(Interrupt::GamePak);
    lgba::irq::enable(EnumSet::all());

    let mut is_odd = false;
    let vcounter_chain = pin!(InterruptHandler::new(|| {
        if is_odd {
            lgba::display::set_counter_scanline(200);
            is_odd = false;
        } else {
            lgba::display::set_counter_scanline(0);
            is_odd = true;
        }
    }));
    vcounter_chain.register(Interrupt::VCounter);

    let mut timer0 = TimerId::Timer0.create();
    timer0
        .set_interrupt_enabled(true)
        .set_timer_mode(TimerMode::Cycle1)
        .set_enabled(true);
    let mut timer1 = TimerId::Timer1.create();
    timer1
        .set_interrupt_enabled(true)
        .set_overflow_at(100)
        .set_timer_mode(TimerMode::Cascade)
        .set_enabled(true);
    let mut timer2 = TimerId::Timer2.create();
    timer2
        .set_interrupt_enabled(true)
        .set_overflow_at(100)
        .set_timer_mode(TimerMode::Cascade)
        .set_enabled(true);
    let mut timer3 = TimerId::Timer3.create();
    timer3
        .set_interrupt_enabled(true)
        .set_overflow_at(100)
        .set_timer_mode(TimerMode::Cascade)
        .set_enabled(true);

    loop {
        lgba::sys::wait_for_vblank();

        for (i, int) in EnumSet::<Interrupt>::all().iter().enumerate() {
            terminal.set_cursor(0, i);
            terminal.reset_line();

            write!(terminal.write(), "{:?}", int);
            terminal.set_cursor(16, i);
            terminal.write_str(":");
            terminal.set_cursor(19, i);
            write!(terminal.write(), "{}", interrupt_count[int as usize].read());
        }
    }
}
