use lgba::{
    display::{Terminal, TerminalFontBasic},
    dma::DmaChannelId,
    sys::Button,
};

pub fn run() -> ! {
    let mut terminal = Terminal::new();
    terminal.use_dma_channel(DmaChannelId::Dma3);
    terminal.set_force_blank(true);
    let terminal = terminal.activate::<TerminalFontBasic>();
    let mut terminal = terminal.lock();

    let cycles = lgba::timer::time_cycles(|| {
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
        {
            let mut write = terminal.write();
            for i in 1..=100 {
                if i % 15 == 0 {
                    write.set_half_width(true);
                    write.write_str("[fizz-buzz]");
                } else if i % 3 == 0 {
                    write.set_half_width(true);
                    write.write_str("[fizz]");
                } else if i % 5 == 0 {
                    write.set_half_width(true);
                    write.write_str("[buzz]");
                } else {
                    write.set_half_width(false);
                    write!(write, "{i}");
                }
                write.set_half_width(true);
                write.write_str(" ");
            }
        }

        terminal.new_line();
    });
    println!("Terminal screen rendered in {} cycles.", cycles);

    lgba::sys::wait_for_vblank();
    terminal.set_force_blank(false);

    let mut frame = 0;
    loop {
        lgba::sys::wait_for_vblank();

        terminal.clear_line(18);
        terminal.set_cursor(0, 18);
        terminal.set_half_width(false);

        let keys = lgba::sys::pressed_keys();
        write!(terminal.write(), "#{frame:03} / ");
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
    }
}
