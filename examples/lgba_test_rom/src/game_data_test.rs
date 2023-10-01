use lgba::{
    display::{Terminal, TerminalFontBasic},
    dma::DmaChannelId,
};

lgba_data::load_data!(RomData, "RomData.toml");

pub fn run() -> ! {
    let mut terminal = Terminal::new();
    terminal.use_dma_channel(DmaChannelId::Dma3);
    terminal.set_force_blank(true);
    let terminal = terminal.activate::<TerminalFontBasic>();
    let mut terminal = terminal.lock();

    let cycles = lgba::timer::time_cycles(|| {
        writeln!(
            terminal.write(),
            "testing strings: {:?} {:?}",
            lgba_data::StrHash!("test string"),
            lgba_data::StrHash::new("test string")
        );
        terminal.write_str(core::str::from_utf8(RomData.test(1, 1).as_slice()).unwrap());
        terminal.write_str(core::str::from_utf8(RomData.test(10, 10).as_slice()).unwrap());
    });
    println!("Terminal screen rendered in {} cycles.", cycles);

    lgba::sys::wait_for_vblank();
    terminal.set_force_blank(false);

    loop {}
}
