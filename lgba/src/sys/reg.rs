use core::marker::PhantomData;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum SafeReg {}
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum UnsafeReg {}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Register<T: Copy, S = SafeReg>(*mut T, PhantomData<S>);
impl<T: Copy, S> Register<T, S> {
    pub const unsafe fn new(offset: usize) -> Self {
        Register(offset as *mut T, PhantomData)
    }
}
impl<T: Copy> Register<T, SafeReg> {
    pub fn write(&self, t: T) {
        unsafe { self.0.write_volatile(t) }
    }
    pub fn read(&self) -> T {
        unsafe { self.0.read_volatile() }
    }
}
impl<T: Copy> Register<T, UnsafeReg> {
    pub unsafe fn assert_safe(&self) -> Register<T, SafeReg> {
        Register(self.0, PhantomData)
    }
    pub unsafe fn write(&self, t: T) {
        self.0.write_volatile(t)
    }
    pub unsafe fn read(&self) -> T {
        self.0.read_volatile()
    }
}
