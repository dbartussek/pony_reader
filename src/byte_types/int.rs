use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
};
use zerocopy::{AsBytes, FromBytes, Unaligned};

macro_rules! define_int {
    ($name:ident, $bytes:expr, $ty:ty, $read: ident, $write: ident, $ty_str:expr) => {
        #[repr(C)]
        #[derive(Default, Copy, Clone, Serialize, Deserialize)]
        #[serde(into = $ty_str)]
        #[serde(from = $ty_str)]
        pub struct $name<B>(pub [u8; $bytes], pub PhantomData<B>)
        where
            B: ByteOrder;

        impl<B> $name<B>
        where
            B: ByteOrder,
        {
            pub fn get(self) -> $ty {
                B::$read(&self.0)
            }

            pub fn set(&mut self, value: $ty) {
                B::$write(&mut self.0, value)
            }
        }

        impl<B> From<$ty> for $name<B>
        where
            B: ByteOrder,
        {
            fn from(value: $ty) -> Self {
                let mut result = Self::default();
                result.set(value);
                result
            }
        }
        impl<B> Into<$ty> for $name<B>
        where
            B: ByteOrder,
        {
            fn into(self) -> $ty {
                self.get()
            }
        }


        impl<B> Debug for $name<B>
        where
            B: ByteOrder,
        {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                Debug::fmt(&self.get(), f)
            }
        }
        impl<B> Display for $name<B>
        where
            B: ByteOrder,
        {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.get(), f)
            }
        }

        unsafe impl<B> FromBytes for $name<B>
        where
            B: ByteOrder,
        {
            fn only_derive_is_allowed_to_implement_this_trait()
            where
                Self: Sized,
            {
            }
        }
        unsafe impl<B> AsBytes for $name<B>
        where
            B: ByteOrder,
        {
            fn only_derive_is_allowed_to_implement_this_trait()
            where
                Self: Sized,
            {
            }
        }
        unsafe impl<B> Unaligned for $name<B>
        where
            B: ByteOrder,
        {
            fn only_derive_is_allowed_to_implement_this_trait()
            where
                Self: Sized,
            {
            }
        }
    };
}

define_int!(U16, 2, u16, read_u16, write_u16, "u16");
define_int!(U32, 4, u32, read_u32, write_u32, "u32");
define_int!(U64, 8, u64, read_u64, write_u64, "u64");

define_int!(I16, 2, i16, read_i16, write_i16, "i16");
define_int!(I32, 4, i32, read_i32, write_i32, "i32");
define_int!(I64, 8, i64, read_i64, write_i64, "i64");
