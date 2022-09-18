mod prelude {
    pub use crate::sys::reg::*;
    pub use core::{
        marker::PhantomData,
        mem::MaybeUninit,
        ops::{Deref, RangeInclusive},
    };
}

macro_rules! packed_struct_read {
    (@num $target:ty, $self:ident, $data:expr, $inner_ty:ty) => {{
        let range: RangeInclusive<usize> = $data;
        let bit_size: usize = *range.end() - *range.start() + 1;
        let mask: $inner_ty = ((1 << bit_size) - 1) as $inner_ty;

        (($self.0 >> *range.start()) & mask) as $target
    }};
    (u8, $self:ident, $data:expr, $inner_ty:ty) => {
        packed_struct_read!(@num u8, $self, $data, $inner_ty)
    };
    (u16, $self:ident, $data:expr, $inner_ty:ty) => {
        packed_struct_read!(@num u16, $self, $data, $inner_ty)
    };
    (u32, $self:ident, $data:expr, $inner_ty:ty) => {
        packed_struct_read!(@num u32, $self, $data, $inner_ty)
    };
    (usize, $self:ident, $data:expr, $inner_ty:ty) => {
        packed_struct_read!(@num usize, $self, $data, $inner_ty)
    };
    (bool, $self:ident, $data:expr, $inner_ty:ty) => {
        $self.0 & (1 << $data) != 0
    };
    // Assume this is an enum.
    ($target:ty, $self:ident, $data:expr, $inner_ty:ty) => {{
        let data = packed_struct_read!(@num $inner_ty, $self, $data, $inner_ty);
        match <$target>::try_from(data) {
            Ok(v) => v,
            Err(_) => $crate::sys::invalid_enum_in_register(),
        }
    }};
}
macro_rules! packed_struct_write {
    (@num $target:ty, $self:ident, $data:expr, $inner_ty:ty, $value:expr) => {{
        let range: RangeInclusive<usize> = $data;
        let bit_size: usize = *range.end() - *range.start() + 1;
        let mask: $inner_ty = ((1 << bit_size) - 1) as $inner_ty;

        if ($value as usize) >= (1 << bit_size) {
            $crate::sys::value_out_of_range($value as usize, 1 << bit_size)
        }
        $self.0 &= !(mask << *range.start());
        $self.0 |= ($value as $inner_ty) << *range.start();
    }};
    (u8, $self:ident, $data:expr, $inner_ty:ty, $value:expr) => {
        packed_struct_write!(@num u8, $self, $data, $inner_ty, $value)
    };
    (u16, $self:ident, $data:expr, $inner_ty:ty, $value:expr) => {
        packed_struct_write!(@num u16, $self, $data, $inner_ty, $value)
    };
    (u32, $self:ident, $data:expr, $inner_ty:ty, $value:expr) => {
        packed_struct_write!(@num u32, $self, $data, $inner_ty, $value)
    };
    (usize, $self:ident, $data:expr, $inner_ty:ty, $value:expr) => {
        packed_struct_write!(@num usize, $self, $data, $inner_ty, $value)
    };
    (bool, $self:ident, $data:expr, $inner_ty:ty, $value:expr) => {
        $self.0 &= !(1 << $data);
        $self.0 |= ($value as $inner_ty) << $data;
    };
    // Assume this is an enum.
    ($target:ty, $self:ident, $data:expr, $inner_ty:ty, $value:expr) => {
        packed_struct_write!(@num $inner_ty, $self, $data, $inner_ty, $value as $inner_ty)
    };
}
macro_rules! packed_struct_fields {
    (
        $name:ident, $setter_name:ident, $inner_ty:ty
        $(
            , $(#[$field_meta:meta])*
            (
                $field_name:ident, $set_field_name:ident, $with_field_name:ident,
                $field_ty:tt, $data:expr $(,)?
            )
        )*
        $(,)?
    ) => {
        #[doc = "A helper for setting registers containing a"]
        #[doc = concat!("[`", stringify!($name), "`].")]
        pub struct $setter_name<'a, const IS_READ: bool = true> {
            register: Register<$name, SafeReg>,
            value: MaybeUninit<$name>,
            _phantom: PhantomData<&'a mut ()>,
        }

        // Initial state
        impl<'a> $setter_name<'a, { false }> {
            pub(crate) fn from_reg(reg: Register<$name, SafeReg>) -> Self {
                $setter_name {
                    register: reg,
                    value: MaybeUninit::uninit(),
                    _phantom: PhantomData,
                }
            }
            pub(crate) unsafe fn from_unsafe_reg(reg: Register<$name, UnsafeReg>) -> Self {
                $setter_name {
                    register: reg.assert_safe(),
                    value:  MaybeUninit::uninit(),
                    _phantom: PhantomData,
                }
            }
        }

        impl<'a, const IS_READ: bool> $setter_name<'a, { IS_READ }> {
            /// Reads the register into memory, if it is not already read.
            ///
            /// This returns a version of the accessor that allows reading the current value of
            /// the field.
            pub fn read(self) -> $setter_name<'a, { true }> {
                let new_value = if IS_READ {
                    unsafe { self.value.assume_init() }
                } else {
                    self.register.read()
                };
                $setter_name {
                    register: self.register,
                    value: MaybeUninit::new(new_value),
                    _phantom: PhantomData,
                }
            }

            /// Resets the uncommitted register value to its default.
            pub fn clear(self) -> $setter_name<'a, { true }> {
                $setter_name {
                    register: self.register,
                    value: MaybeUninit::new(Default::default()),
                    _phantom: PhantomData,
                }
            }

            /// Writes the uncommitted register value to the IO port.
            pub fn commit(self) {
                let read = self.read();
                read.register.write(unsafe { read.value.assume_init() });
            }

            $(
                #[doc = "Sets the"]
                #[doc = concat!(
                    "[`", stringify!($field_name), "`]",
                    "(`", stringify!($name), "::", stringify!($field_name), "`)",
                )]
                #[doc = "field of the uncommitted register value."]
                pub fn $set_field_name(self,value: $field_ty) -> $setter_name<'a, { true }> {
                    let read = self.read();
                    let new_value = unsafe { read.value.assume_init().$with_field_name(value) };
                    $setter_name {
                        register: read.register,
                        value: MaybeUninit::new(new_value),
                        _phantom: PhantomData,
                    }
                }
            )*
        }
        impl<'a> Deref for $setter_name<'a, { true }> {
            type Target = $name;
            fn deref(&self) -> &Self::Target {
                unsafe { self.value.assume_init_ref() }
            }
        }

        impl $name {
            $(
                $(#[$field_meta])*
                pub fn $field_name(&self) -> $field_ty {
                    packed_struct_read!($field_ty, self, $data, $inner_ty)
                }
            )*

            $(
                #[doc = "Sets the"]
                #[doc = concat!(
                    "[`", stringify!($field_name), "`]",
                    "(`", stringify!($name), "::", stringify!($field_name), "`)",
                )]
                #[doc = "field."]
                pub fn $with_field_name(mut self, value: $field_ty) -> Self {
                    packed_struct_write!($field_ty, self, $data, $inner_ty, value);
                    self
                }
            )*
        }
    };
}

#[inline(never)]
fn invalid_enum_in_register() -> ! {
    panic!("invalid enum value in io register??")
}
#[inline(never)]
fn value_out_of_range(max: usize, value: usize) -> ! {
    panic!("value for field exceeds limits: {value} >= {max}")
}

mod reg;

pub mod lcd;
