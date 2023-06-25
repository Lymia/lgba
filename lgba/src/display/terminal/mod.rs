use crate::{
    display::{
        modes::{ActiveMode0, Mode0},
        vram::MapAccess,
        TerminalFont, VramTile,
    },
    dma::DmaChannelId,
    mmio::reg::BG_PALETTE_RAM,
    sync::{Mutex, MutexGuard, Static},
};
use core::{fmt, marker::PhantomData};

pub mod fonts;

fn update_palette(id: usize, (background, foreground): (u16, u16)) {
    let mut start = id * 0x40;
    for i in 0..4 {
        let mask = 1 << (3 - i);
        for j in start..start + 0x10 {
            let color = if (j & mask) != 0 { foreground } else { background };
            BG_PALETTE_RAM.index(j).write(color);
        }
        start += 0x10;
    }
}

/// A terminal display mode that makes it easy to display text.
pub struct Terminal {
    mode: Mode0,
    dma_channel: Option<DmaChannelId>,
    terminal_colors: [Static<(u16, u16)>; 4],
}
impl Terminal {
    pub fn new() -> Self {
        Terminal {
            mode: Mode0::new(),
            dma_channel: None,
            terminal_colors: [
                Static::new((0, !0)),
                Static::new((0, !0)),
                Static::new((0, !0)),
                Static::new((0, !0)),
            ],
        }
    }

    #[track_caller]
    pub fn use_dma_channel(&mut self, id: DmaChannelId) {
        if id.is_source_internal_only() {
            terminal_invalid_dma_channel();
        }
        self.dma_channel = Some(id);
    }

    pub fn set_color(&mut self, id: usize, background: u16, foreground: u16) {
        if id >= 4 {
            terminal_color_out_of_range();
        }
        self.terminal_colors[id].write((background, foreground));
        update_palette(id, self.terminal_colors[id].read());
    }

    pub fn set_force_blank(&mut self, force_blank: bool) {
        self.mode.set_force_blank(force_blank);
    }

    fn active_raw<T: TerminalFont>(&mut self, no_lock: bool) -> ActiveTerminal<T> {
        // configure all layers
        self.mode.layers[0]
            .set_enabled(true)
            .set_tile_base(28)
            .set_h_offset(4)
            .set_v_offset(4);
        self.mode.layers[1]
            .set_enabled(true)
            .set_tile_base(29)
            .set_h_offset(8)
            .set_v_offset(4);
        self.mode.layers[2]
            .set_enabled(true)
            .set_tile_base(30)
            .set_h_offset(4)
            .set_v_offset(4);
        self.mode.layers[3]
            .set_enabled(true)
            .set_tile_base(31)
            .set_h_offset(8)
            .set_v_offset(4);

        // upload font
        if let Some(channel) = self.dma_channel {
            self.mode.layers[0].char_access().write_char_4bpp_dma(
                channel.create(),
                0,
                T::get_font_data(),
            );
        } else {
            self.mode.layers[0]
                .char_access()
                .write_char_4bpp(0, T::get_font_data());
        }

        // upload palette
        for i in 0..4 {
            update_palette(i, self.terminal_colors[i].read());
        }

        // create an appropriate active terminal object
        let active_mode = if no_lock {
            self.mode.activate_no_lock()
        } else {
            self.mode.activate()
        };
        let map = [
            active_mode.layers[0].map_access(0),
            active_mode.layers[1].map_access(0),
            active_mode.layers[2].map_access(0),
            active_mode.layers[3].map_access(0),
        ];
        let terminal = ActiveTerminal {
            state: Mutex::new(ActiveTerminalState {
                cursor_x: 0,
                cursor_y: 0,
                cursor_hw: false,
                color: 0,
                line_advance: 0,
                mode: active_mode,
                terminal_colors: &self.terminal_colors,
                map,
                space_ch: [
                    ActiveTerminalAccess::<T>::tile_for_ch('\u{F508}', 0),
                    ActiveTerminalAccess::<T>::tile_for_ch('\u{F508}', 1),
                    ActiveTerminalAccess::<T>::tile_for_ch('\u{F508}', 2),
                    ActiveTerminalAccess::<T>::tile_for_ch('\u{F508}', 3),
                ],
                half_bg_ch: [
                    ActiveTerminalAccess::<T>::tile_for_ch('\u{F505}', 0),
                    ActiveTerminalAccess::<T>::tile_for_ch('\u{F505}', 1),
                    ActiveTerminalAccess::<T>::tile_for_ch('\u{F505}', 2),
                    ActiveTerminalAccess::<T>::tile_for_ch('\u{F505}', 3),
                ],
                full_bg_ch: [
                    ActiveTerminalAccess::<T>::tile_for_ch('\u{F501}', 0),
                    ActiveTerminalAccess::<T>::tile_for_ch('\u{F501}', 1),
                    ActiveTerminalAccess::<T>::tile_for_ch('\u{F501}', 2),
                    ActiveTerminalAccess::<T>::tile_for_ch('\u{F501}', 3),
                ],
                dma_channel: self.dma_channel,
            }),
            _phantom: Default::default(),
        };

        // clear the entire terminal
        terminal.clear();

        // return the terminal
        terminal
    }

    pub fn activate<T: TerminalFont>(&mut self) -> ActiveTerminal<T> {
        self.active_raw(false)
    }
    pub fn activate_no_lock<T: TerminalFont>(&mut self) -> ActiveTerminal<T> {
        self.active_raw(true)
    }
}

/// An active terminal display mode.
pub struct ActiveTerminal<'a, T: TerminalFont> {
    state: Mutex<ActiveTerminalState<'a>>,
    _phantom: PhantomData<T>,
}
impl<'a, T: TerminalFont> ActiveTerminal<'a, T> {
    /// Locks this active terminal ahead of time, allowing for faster terminal access.
    pub fn lock<'x>(&'x self) -> ActiveTerminalAccess<'x, 'a, T>
    where 'a: 'x {
        ActiveTerminalAccess {
            term: match self.state.try_lock() {
                None => terminal_lock_failure(),
                Some(x) => x,
            },
            _phantom: Default::default(),
        }
    }

    /// Sets the color for this terminal.
    #[track_caller]
    pub fn set_color(&self, id: usize, background: u16, foreground: u16) {
        self.lock().set_color(id, background, foreground)
    }
    pub fn clear(&self) {
        self.lock().clear();
    }

    #[track_caller]
    pub fn set_cursor(&self, x: usize, y: usize) {
        self.lock().set_cursor(x, y);
    }

    pub fn set_cursor_half_width(&self, half_width: bool) {
        self.lock().set_half_width(half_width);
    }

    #[track_caller]
    pub fn set_char_full(&self, x: usize, y: usize, ch: char, color: usize) {
        self.lock().set_char_full(x, y, ch, color);
    }

    #[track_caller]
    pub fn set_char_half(&self, x: usize, y: usize, ch: char, color: usize) {
        self.lock().set_char_half(x, y, ch, color);
    }
}

struct ActiveTerminalState<'a> {
    cursor_x: u8,
    cursor_y: u8,
    cursor_hw: bool,
    color: u8,
    line_advance: u8,

    mode: ActiveMode0<'a>,
    terminal_colors: &'a [Static<(u16, u16)>; 4],
    map: [MapAccess; 4],

    space_ch: [VramTile; 4],
    half_bg_ch: [VramTile; 4],
    full_bg_ch: [VramTile; 4],

    dma_channel: Option<DmaChannelId>,
}
impl<'a> ActiveTerminalState<'a> {
    fn apply_advance(&self, y: usize) -> usize {
        (y + (self.line_advance as usize)) % 32
    }
    #[track_caller]
    fn check_coordinate(x: usize, y: usize) {
        if x >= 58 && y >= 19 {
            terminal_coord_out_of_range();
        }
    }

    fn set_color(&self, id: usize, background: u16, foreground: u16) {
        self.terminal_colors[id].write((background, foreground));
        update_palette(id, self.terminal_colors[id].read());
    }
    fn clear(&mut self) {
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.color = 0;
        self.line_advance = 0;

        let tile = self.space_ch[0];
        if let Some(channel) = self.dma_channel {
            for i in 0..4 {
                self.map[i].set_tile_dma(channel.create(), 0, 0, tile, 32 * 32);
            }
        } else {
            for i in 0..4 {
                for x in 0..32 {
                    for y in 0..32 {
                        self.map[i].set_tile(x, y, tile);
                    }
                }
            }
        }
    }

    #[track_caller]
    fn set_char_full(&self, x: usize, y: usize, tile: VramTile, color: usize) {
        Self::check_coordinate(x * 2, y);
        let y = self.apply_advance(y);

        self.map[0].set_tile(x, y, tile);
        self.map[1].set_tile(x, y, self.space_ch[color]);
        self.map[2].set_tile(x, y, self.full_bg_ch[color]);
    }
    #[track_caller]
    fn set_char_half(&self, x: usize, y: usize, (tile, is_half): (VramTile, bool), color: usize) {
        Self::check_coordinate(x, y);
        let y = self.apply_advance(y);

        let (plane, tile_x) = (x % 2, x / 2);
        if is_half && plane == 0 {
            self.map[0].set_tile(tile_x, y, tile);
            self.map[2].set_tile(tile_x, y, self.half_bg_ch[color]);
        } else if is_half {
            self.map[1].set_tile(tile_x, y, tile);
            self.map[3].set_tile(tile_x, y, self.half_bg_ch[color]);
        } else if plane == 0 {
            self.map[0].set_tile(tile_x, y, tile);
            self.map[1].set_tile(tile_x, y, self.space_ch[color]);
            self.map[2].set_tile(tile_x, y, self.full_bg_ch[color]);
        } else {
            self.map[0].set_tile(tile_x + 1, y, self.space_ch[color]);
            self.map[1].set_tile(tile_x, y, tile);
            self.map[2].set_tile(tile_x + 1, y, self.space_ch[color]);
            self.map[3].set_tile(tile_x, y, self.full_bg_ch[color]);
        }
    }

    #[track_caller]
    fn set_cursor(&mut self, x: usize, y: usize) {
        Self::check_coordinate(x, y);
        self.cursor_x = x as u8;
        self.cursor_y = y as u8;
    }
    fn set_half_width(&mut self, half_width: bool) {
        self.cursor_hw = half_width;
    }

    fn clear_line(&mut self, y: usize) {
        Self::check_coordinate(0, y);
        let y = self.apply_advance(y);

        for i in 0..4 {
            for x in 0..29 {
                self.map[i].set_tile(x, y, self.space_ch[0]);
            }
        }
    }
    fn advance_screen(&mut self) {
        self.clear_line(0);

        self.line_advance += 1;
        if self.line_advance == 32 {
            self.line_advance = 0;
        }

        for i in 0..4 {
            self.mode.layers[i].set_v_offset(4 - self.line_advance as i16 * 8);
        }
    }
    fn advance_cursor(&mut self) {
        self.cursor_x = 0;
        if self.cursor_y == 18 {
            self.advance_screen();
        } else {
            self.cursor_y += 1;
        }
    }
}

/// A lock on an [`ActiveTerminal`], allowing for faster terminal writes.
pub struct ActiveTerminalAccess<'a, 'b: 'a, T: TerminalFont> {
    term: MutexGuard<'a, ActiveTerminalState<'b>>,
    _phantom: PhantomData<T>,
}
impl<'a, 'b: 'a, T: TerminalFont> ActiveTerminalAccess<'a, 'b, T> {
    pub fn set_force_blank(&mut self, force_blank: bool) {
        self.term.mode.set_force_blank(force_blank);
    }

    #[track_caller]
    pub fn set_color(&self, id: usize, background: u16, foreground: u16) {
        if id >= 4 {
            terminal_color_out_of_range();
        }
        self.term.set_color(id, background, foreground);
    }

    #[track_caller]
    fn data_for_ch(ch: char, color: usize) -> (VramTile, bool) {
        if color >= 4 {
            terminal_color_out_of_range();
        }
        let (plane, tile, is_half) = T::get_font_glyph(ch);
        let pal = color as u8 * 4 + plane;
        (VramTile::default().with_char(tile).with_palette(pal as u8), is_half)
    }
    #[track_caller]
    fn tile_for_ch(ch: char, color: usize) -> VramTile {
        Self::data_for_ch(ch, color).0
    }

    pub fn clear(&mut self) {
        self.term.clear();
    }
    pub fn clear_line(&mut self, y: usize) {
        self.term.clear_line(y);
    }

    fn process_hw(ch: char) -> char {
        if ch.is_ascii() {
            unsafe { char::from_u32_unchecked((ch as u32) + 0xF400) }
        } else {
            ch
        }
    }
    fn process_hw_conditional(&self, ch: char) -> char {
        if self.term.cursor_hw {
            Self::process_hw(ch)
        } else {
            ch
        }
    }

    #[track_caller]
    pub fn set_char_full(&self, x: usize, y: usize, ch: char, color: usize) {
        let tile = Self::tile_for_ch(ch, color);
        self.term.set_char_full(x, y, tile, color);
    }
    #[track_caller]
    pub fn set_char_half(&self, x: usize, y: usize, ch: char, color: usize) {
        let ch = Self::process_hw(ch);
        self.term
            .set_char_half(x, y, Self::data_for_ch(ch, color), color);
    }

    #[track_caller]
    pub fn set_cursor(&mut self, x: usize, y: usize) {
        self.term.set_cursor(x, y);
    }

    #[track_caller]
    pub fn set_active_color(&mut self, color: usize) {
        if color >= 4 {
            terminal_color_out_of_range();
        }
        self.term.color = color as u8;
    }

    pub fn set_half_width(&mut self, half_width: bool) {
        self.term.set_half_width(half_width);
    }

    pub fn new_line(&mut self) {
        self.term.advance_cursor();
    }

    pub fn write_char(&mut self, ch: char) {
        let ch = self.process_hw_conditional(ch);

        let x = self.term.cursor_x as usize;
        let y = self.term.cursor_y as usize;
        let color = self.term.color as usize;

        // TODO: Better handling for edge cases.

        let wrote_hw = if self.term.cursor_hw || self.term.cursor_x % 2 == 1 {
            let data = Self::data_for_ch(ch, color);
            self.term.set_char_half(x, y, data, color);
            self.term.cursor_hw && data.1
        } else {
            self.term
                .set_char_full(x / 2, y, Self::tile_for_ch(ch, color), color);
            false
        };

        self.term.cursor_x += 2 - wrote_hw as u8;
        if self.term.cursor_x >= 57 + self.term.cursor_hw as u8 {
            self.new_line();
        }
    }
    pub fn write<'x>(&'x mut self) -> ActiveTerminalWrite<'x, 'a, 'b, T>
    where
        'a: 'x,
        'b: 'x,
    {
        ActiveTerminalWrite {
            access: self,
            buffer: [0; 60],
            buffer_idx: 0,
            passthrough_mode: false,
        }
    }
    pub fn write_str(&mut self, str: &str) {
        self.write().write_str(str);
    }
}

pub struct ActiveTerminalWrite<'a, 'b: 'a, 'c: 'a + 'b, T: TerminalFont> {
    access: &'a mut ActiveTerminalAccess<'b, 'c, T>,
    buffer: [u16; 60],
    buffer_idx: usize,
    passthrough_mode: bool,
}
impl<'a, 'b: 'a, 'c: 'a + 'b, T: TerminalFont> ActiveTerminalWrite<'a, 'b, 'c, T> {
    fn dump_buffers(&mut self) {
        for ch in &self.buffer[..self.buffer_idx] {
            let ch = unsafe { char::from_u32_unchecked(*ch as u32) };
            self.access.write_char(ch);
        }
    }
    fn flush_buffers(&mut self) {
        if self.passthrough_mode {
            self.passthrough_mode = false;
        } else {
            let fits_on_line = self.buffer_idx < (58 - self.access.term.cursor_x as usize);
            let fits_on_next_line = self.buffer_idx < 58;
            if !fits_on_line && fits_on_next_line {
                self.access.new_line();
            }
            self.dump_buffers();
        }
        self.buffer_idx = 0;
    }
    fn push_char(&mut self, ch: char) {
        if self.passthrough_mode {
            self.access.write_char(ch);
        } else if self.buffer_idx == 60 {
            self.passthrough_mode = true;
            self.dump_buffers();
            self.access.write_char(ch);
        } else {
            let glyph = if ch as u32 >= 0x10000 {
                0xF50F // guaranteed non-existant
            } else {
                ch as u16
            };
            self.buffer[self.buffer_idx] = glyph;
            self.buffer_idx += 1;
        }
    }
    fn write_char(&mut self, ch: char) {
        match ch {
            ' ' => {
                self.flush_buffers();
                if self.access.term.cursor_x != 0 {
                    self.access.write_char(' ');
                }
            }
            '\t' => {
                self.flush_buffers();

                if !self.access.term.cursor_hw && self.access.term.cursor_x % 2 != 0 {
                    self.access.term.cursor_hw = true;
                    self.access.write_char(' ');
                    self.access.term.cursor_hw = false;
                }
                while self.access.term.cursor_x % 8 == 0 {
                    self.access.write_char(' ');
                }
            }
            '\n' => {
                self.flush_buffers();
                self.access.new_line();
            }
            _ => self.push_char(ch),
        }
    }
    fn write_str(&mut self, s: &str) {
        for char in s.chars() {
            self.write_char(char);
        }
    }
}
impl<'a, 'b: 'a, 'c: 'a + 'b, T: TerminalFont> fmt::Write for ActiveTerminalWrite<'a, 'b, 'c, T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.write_char(c);
        Ok(())
    }
}
impl<'a, 'b: 'a, 'c: 'a + 'b, T: TerminalFont> Drop for ActiveTerminalWrite<'a, 'b, 'c, T> {
    fn drop(&mut self) {
        self.flush_buffers();
    }
}

#[inline(never)]
#[track_caller]
fn terminal_coord_out_of_range() -> ! {
    crate::panic_handler::static_panic(
        "Terminal coordinates are from (0,0) to (57,18) inclusive in half-width mode and between \
        (0,0) to (28,18) inclusive in full-width mode.",
    )
}

#[inline(never)]
#[track_caller]
fn terminal_color_out_of_range() -> ! {
    crate::panic_handler::static_panic("Terminals only support up to 4 colors (0-3)!")
}

#[inline(never)]
#[track_caller]
fn terminal_lock_failure() -> ! {
    crate::panic_handler::static_panic("Terminal is already locked!")
}

#[inline(never)]
#[track_caller]
fn terminal_invalid_dma_channel() -> ! {
    crate::panic_handler::static_panic("DMA channel cannot be used for terminal rendering!")
}
