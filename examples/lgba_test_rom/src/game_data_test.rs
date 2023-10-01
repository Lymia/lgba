use lgba::{
    display::{Terminal, TerminalFontBasic},
    dma::DmaChannelId,
};

lgba_data::load_data!(DataTest, "DataTest.toml");

pub fn run() -> ! {
    let mut terminal = Terminal::new();
    terminal.use_dma_channel(DmaChannelId::Dma3);
    terminal.set_force_blank(true);
    let terminal = terminal.activate::<TerminalFontBasic>();
    let mut terminal = terminal.lock();

    let cycles = lgba::timer::time_cycles(|| {
        terminal.write_str(core::str::from_utf8(DataTest.test(1, 1).as_slice()).unwrap());
    });
    println!("Terminal screen rendered in {} cycles.", cycles);

    lgba::sys::wait_for_vblank();
    terminal.set_force_blank(false);

    loop {}
}
