use core::{cell::UnsafeCell, marker::PhantomData};
#[cfg(feature = "generator_build")]
use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[repr(C)]
pub struct ExHeader<T: ExHeaderType> {
    magic: [u8; 4],
    name: [u8; 4],
    version: u16,
    length: u16,
    pub data: UnsafeCell<T>, // represent out-of-compiler changes with UnsafeCell
}
impl<T: ExHeaderType> ExHeader<T> {
    pub const fn new(data: T) -> Self {
        assert!(core::mem::size_of::<T>() <= u16::MAX as usize);
        ExHeader {
            magic: *b"lGex",
            name: T::NAME,
            version: T::VERSION,
            length: core::mem::size_of::<T>() as u16,
            data: UnsafeCell::new(data),
        }
    }

    pub const fn name(&self) -> [u8; 4] {
        self.name
    }

    pub const fn version(&self) -> u16 {
        self.version
    }
}
unsafe impl<T: ExHeaderType + Sync> Sync for ExHeader<T> {}
unsafe impl<T: ExHeaderType + Send> Send for ExHeader<T> {}

pub trait ExHeaderType {
    const NAME: [u8; 4];
    const VERSION: u16;
}

#[cfg_attr(feature = "generator_build", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct SerialSlice<T> {
    pub ptr: u32,
    pub len: u32,
    pub _phantom: PhantomData<T>,
}
impl<T> SerialSlice<T> {
    pub const fn new(ptr: usize, len: usize) -> Self {
        assert!((ptr as u64) < u32::MAX as u64);
        assert!((len as u64) < u32::MAX as u64);
        SerialSlice { ptr: ptr as u32, len: len as u32, _phantom: PhantomData }
    }
    pub const fn default() -> Self {
        SerialSlice::new(0, 0)
    }

    pub unsafe fn offset(&self, idx: usize) -> *const T {
        if idx >= self.len as usize {
            slice_range_fail(idx, self.len);
        }
        let raw_ptr = self.ptr as *const T;
        raw_ptr.offset(idx as isize)
    }

    pub unsafe fn as_slice(&self) -> &'static [T] {
        core::slice::from_raw_parts(self.ptr as *const T, self.len as usize)
    }
}
impl<T> Default for SerialSlice<T> {
    fn default() -> Self {
        SerialSlice::new(0, 0)
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct StaticStr {
    pub ptr: *const u8,
    pub len: usize,
}
impl StaticStr {
    pub const fn new(v: &'static str) -> StaticStr {
        StaticStr { ptr: v.as_ptr(), len: v.len() }
    }

    pub unsafe fn as_str(&self) -> &'static str {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(self.ptr, self.len))
    }
}
unsafe impl Send for StaticStr {}
unsafe impl Sync for StaticStr {}

#[inline(never)]
fn slice_range_fail(offset: usize, len: u32) -> ! {
    panic!("offset longer than SerialSlice length. {offset} >= {len}")
}
