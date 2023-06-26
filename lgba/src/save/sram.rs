//! Module for battery backed SRAM save media support.
//!
//! SRAM acts as ordinary memory mapped into the memory space, and as such
//! is accessed using normal memory read/write commands.

use crate::{
    mmio::sys::WaitState,
    save::{utils::Timeout, Error, MediaInfo, MediaType, RawSaveAccess},
};

const SRAM_SIZE: usize = 32 * 1024; // 32 KiB

/// Checks whether an offset is contained within the bounds of the SRAM.
fn check_bounds(offset: usize, len: usize) -> Result<(), Error> {
    if offset.checked_add(len).is_none() || offset + len > SRAM_SIZE {
        return Err(Error::OutOfBounds);
    }
    Ok(())
}

/// The [`RawSaveAccess`] used for battery backed SRAM.
pub struct BatteryBackedAccess;
impl RawSaveAccess for BatteryBackedAccess {
    fn on_create(&self) {
        crate::save::utils::set_sram_wait(WaitState::Wait8);
    }

    fn info(&self) -> Result<&'static MediaInfo, Error> {
        Ok(&MediaInfo {
            media_type: MediaType::Sram32K,
            sector_shift: 0,
            sector_count: SRAM_SIZE,
            minimum_fast_alignment: 1,
            minimum_acceptable_alignment: 1,
            uses_prepare_write: false,
        })
    }

    fn read(&self, offset: usize, buffer: &mut [u8], _: &mut Timeout) -> Result<(), Error> {
        check_bounds(offset, buffer.len())?;
        unsafe {
            crate::asm::sram_read_raw_buf(buffer, 0x0E000000 + offset);
        }
        Ok(())
    }

    fn verify(&self, offset: usize, buffer: &[u8], _: &mut Timeout) -> Result<bool, Error> {
        check_bounds(offset, buffer.len())?;
        let val = unsafe { crate::asm::sram_verify_raw_buf(buffer, 0x0E000000 + offset) };
        Ok(val)
    }

    fn prepare_write(&self, _: usize, _: usize, _: &mut Timeout) -> Result<(), Error> {
        Ok(())
    }

    fn write(&self, offset: usize, buffer: &[u8], _: &mut Timeout) -> Result<(), Error> {
        check_bounds(offset, buffer.len())?;
        unsafe {
            crate::asm::sram_write_raw_buf(0x0E000000 + offset, buffer);
        }
        Ok(())
    }
}
