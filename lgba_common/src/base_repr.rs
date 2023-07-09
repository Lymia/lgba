use core::marker::PhantomData;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct ExHeader<T: ExHeaderType> {
    pub magic: [u8; 4],
    pub name: [u8; 4],
    pub version: u16,
    pub length: u16,
    pub data: T,
}
impl<T: ExHeaderType> ExHeader<T> {
    pub const fn new(data: T) -> Self {
        assert!(core::mem::size_of::<T>() <= u16::MAX as usize);
        ExHeader {
            magic: *b"lGex",
            name: T::NAME,
            version: T::VERSION,
            length: core::mem::size_of::<T>() as u16,
            data,
        }
    }
}

pub trait ExHeaderType {
    const NAME: [u8; 4];
    const VERSION: u16;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SerialSlice<T> {
    pub ptr: u32,
    pub len: u32,
    pub _phantom: PhantomData<T>,
}
impl<T> SerialSlice<T> {
    pub const fn new(ptr: usize, len: usize) -> Self {
        assert!((ptr as u64) < u32::MAX as u64);
        assert!((len as u64) < u32::MAX as u64);
        SerialSlice {
            ptr: ptr as u32,
            len: len as u32,
            _phantom: PhantomData,
        }
    }
    pub const fn default() -> Self {
        SerialSlice::new(0, 0)
    }

    pub unsafe fn offset(&self, idx: usize) -> *const T {
        assert!(idx < self.len as usize);
        let raw_ptr = self.ptr as *const T;
        raw_ptr.offset(idx as isize)
    }
}
impl<T> Default for SerialSlice<T> {
    fn default() -> Self {
        SerialSlice::new(0, 0)
    }
}