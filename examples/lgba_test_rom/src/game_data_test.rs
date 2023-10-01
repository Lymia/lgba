use lgba::{
    display::{Terminal, TerminalFontBasic},
    dma::DmaChannelId,
};
use lgba_data::StrHash;

lgba_data::load_data!(RomData, "RomData.toml");

pub fn run() -> ! {
    let mut terminal = Terminal::new();
    terminal.use_dma_channel(DmaChannelId::Dma3);
    terminal.set_force_blank(true);
    let terminal = terminal.activate::<TerminalFontBasic>();
    let mut terminal = terminal.lock();

    let bm_0 = lgba::timer::time_cycles(|| {
        core::hint::black_box(RomData.test4(1).as_slice());
    });
    let bm_1 = lgba::timer::time_cycles(|| {
        core::hint::black_box(RomData.test(1, 1).as_slice());
    });
    let bm_2 = lgba::timer::time_cycles(|| {
        core::hint::black_box(RomData.test3(StrHash!("a_1_1")).as_slice());
    });
    let bm_3 = lgba::timer::time_cycles(|| {
        core::hint::black_box(RomData.test3(StrHash!("a_1_1")).as_slice());
        core::hint::black_box(RomData.test3(StrHash!("a_1_2")).as_slice());
        core::hint::black_box(RomData.test3(StrHash!("a_10_10")).as_slice());
        core::hint::black_box(RomData.test3(StrHash!("b_15_15")).as_slice());
    });
    let cycles = lgba::timer::time_cycles(|| {
        terminal.set_half_width(true);
        write!(
            terminal.write(),
            "testing strings: {:?} {:?}",
            StrHash!("test string"),
            StrHash::new("test string")
        );
        terminal.new_line();
        terminal.write_str(core::str::from_utf8(RomData.test4(1).as_slice()).unwrap());
        terminal.new_line();
        terminal.write_str(core::str::from_utf8(RomData.test4(2).as_slice()).unwrap());
        terminal.new_line();
        terminal.write_str(core::str::from_utf8(RomData.test(1, 1).as_slice()).unwrap());
        terminal.new_line();
        terminal.write_str(core::str::from_utf8(RomData.test(10, 10).as_slice()).unwrap());
        terminal.new_line();
        terminal
            .write_str(core::str::from_utf8(RomData.test3(StrHash!("a_1_1")).as_slice()).unwrap());
        terminal.new_line();
        terminal.new_line();
        write!(terminal.write(), "Benchmarks: {bm_0} / {bm_1} / {bm_2} / {bm_3} cycles");
    });
    println!("Terminal screen rendered in {} cycles.", cycles);

    lgba::sys::wait_for_vblank();
    terminal.set_force_blank(false);

    loop {}
}
