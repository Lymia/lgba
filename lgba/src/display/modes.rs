use crate::{
    display::layers::{ActiveTileLayer, LayerId, TileLayer},
    mmio::{display::DispMode, reg::DISPCNT},
    sync::{RawMutex, RawMutexGuard},
};

static MAIN_GFX_LOCK: RawMutex = RawMutex::new();

#[derive(Debug)]
#[non_exhaustive]
pub struct Mode0 {
    pub layers: [TileLayer; 4],
    force_blank: bool,
}
impl Mode0 {
    pub fn new() -> Self {
        Mode0 {
            layers: [
                TileLayer::new(LayerId::Layer0),
                TileLayer::new(LayerId::Layer1),
                TileLayer::new(LayerId::Layer2),
                TileLayer::new(LayerId::Layer3),
            ],
            force_blank: false,
        }
    }

    /// Sets whether the screen is forced to a blank state.
    pub fn set_force_blank(&mut self, force_blank: bool) {
        self.force_blank = force_blank;
    }

    fn activate_raw(&mut self, lock: Option<RawMutexGuard<'static>>) -> ActiveMode0 {
        let [layer0, layer1, layer2, layer3] = &mut self.layers;
        let new_disp_cnt = DISPCNT
            .read()
            .with_mode(DispMode::Mode0)
            .with_forced_blank(self.force_blank)
            .with_display_bg0(layer0.enabled())
            .with_display_bg1(layer1.enabled())
            .with_display_bg2(layer2.enabled())
            .with_display_bg3(layer3.enabled());
        let active_mode = ActiveMode0 {
            layers: [layer0.activate(), layer1.activate(), layer2.activate(), layer3.activate()],
            force_blank: &mut self.force_blank,
            _lock: lock,
        };
        DISPCNT.write(new_disp_cnt);
        active_mode
    }

    /// Activates this mode.
    ///
    /// This checks a global lock to avoid situations where two graphics modes are active at the
    /// same time.
    #[track_caller]
    pub fn activate(&mut self) -> ActiveMode0 {
        let lock = Some(
            MAIN_GFX_LOCK
                .try_lock()
                .unwrap_or_else(|| graphics_in_use()),
        );
        self.activate_raw(lock)
    }

    /// Activates this mode without locking the screen.
    ///
    /// This should not be used except in very special circumstances, such as in a panic handler
    /// that may need to be called in a context where the graphics are already locked.
    ///
    /// There is no risk of memory unsafety while using this, but a great risk of very glitchy
    /// graphics problems.
    pub fn activate_no_lock(&mut self) -> ActiveMode0 {
        self.activate_raw(None)
    }
}

pub struct ActiveMode0<'a> {
    pub layers: [ActiveTileLayer<'a>; 4],
    force_blank: &'a mut bool,
    _lock: Option<RawMutexGuard<'static>>,
}
impl<'a> ActiveMode0<'a> {
    /// Sets whether the screen is forced to a blank state.
    pub fn set_force_blank(&mut self, force_blank: bool) {
        let prev = *self.force_blank;
        *self.force_blank = force_blank;
        if prev != force_blank {
            DISPCNT.write(DISPCNT.read().with_forced_blank(force_blank));
        }
    }
}
impl<'a> Drop for ActiveMode0<'a> {
    fn drop(&mut self) {
        // force blank when there's no active graphics mode
        DISPCNT.write(DISPCNT.read().with_forced_blank(true));
    }
}

#[inline(never)]
#[track_caller]
fn graphics_in_use() -> ! {
    crate::panic_handler::static_panic("A graphics mode is already activated!")
}
