#![allow(unused)] // a lot of the definitions here are unused in various contexts.

mod prelude {
    pub use crate::mmio::reg::*;
    pub use core::{
        marker::PhantomData,
        mem::MaybeUninit,
        ops::{Deref, RangeInclusive},
    };
}

macro_rules! packed_struct_ty {
    ((@enumset $inner_ty:ty)) => {
        enumset::EnumSet<$inner_ty>
    };
    ($ty:ty) => {
        $ty
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
    ((@enumset $target:ty), $self:ident, $data:expr, $inner_ty:ty) => {{
        let data = packed_struct_read!(@num $inner_ty, $self, $data, $inner_ty);
        enumset::EnumSet::try_from_u32(data as u32)
            .unwrap_or_else(|| $crate::mmio::invalid_enum_in_register())
    }};
    // Assume this is an enum.
    ($target:ty, $self:ident, $data:expr, $inner_ty:ty) => {{
        let data = packed_struct_read!(@num $inner_ty, $self, $data, $inner_ty);
        match <$target>::try_from(data) {
            Ok(v) => v,
            Err(_) => $crate::mmio::invalid_enum_in_register(),
        }
    }};
}
macro_rules! packed_struct_write {
    (@num $target:ty, $self:ident, $data:expr, $inner_ty:ty, $value:expr) => {{
        let range: RangeInclusive<usize> = $data;
        let bit_size: usize = *range.end() - *range.start() + 1;
        let mask: $inner_ty = ((1 << bit_size) - 1) as $inner_ty;

        if ($value as usize) >= (1 << bit_size) {
            $crate::mmio::value_out_of_range($value as usize, 1 << bit_size)
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
    ((@enumset $target:ty), $self:ident, $data:expr, $inner_ty:ty, $value:expr) => {
        packed_struct_write!(@num u32, $self, $data, $inner_ty, $value.as_u32_truncated())
    };
    // Assume this is an enum.
    ($target:ty, $self:ident, $data:expr, $inner_ty:ty, $value:expr) => {
        packed_struct_write!(@num $inner_ty, $self, $data, $inner_ty, $value as $inner_ty)
    };
}
macro_rules! packed_struct_fields {
    (
        $name:ident, $inner_ty:ty
        $(
            , $(#[$field_meta:meta])*
            (
                $field_name:ident, $with_field_name:ident,
                $field_ty:tt, $data:expr $(,)?
            )
        )*
        $(,)?
    ) => {
        impl $name {
            $(
                $(#[$field_meta])*
                pub fn $field_name(&self) -> packed_struct_ty!($field_ty) {
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
                pub fn $with_field_name(mut self, value: packed_struct_ty!($field_ty)) -> Self {
                    packed_struct_write!($field_ty, self, $data, $inner_ty, value);
                    self
                }
            )*
        }
    };
}

#[inline(never)]
fn invalid_enum_in_register() -> ! {
    crate::panic_handler::static_panic("invalid enum value in io register??")
}
#[inline(never)]
fn value_out_of_range(max: usize, value: usize) -> ! {
    panic!("value for field exceeds limits: {value} >= {max}")
}

pub mod reg;

pub mod display;
pub mod emulator;
pub mod sys;
