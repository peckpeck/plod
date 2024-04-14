#![deny(missing_docs)]

macro_rules! impl_from {
    ($name:ident, $ty:ty, $from_method:ident, $size:expr) => {
        /// Convert bytes to $ty
        fn $name(bytes: [u8; $size]) -> $ty {
            <$ty>::$from_method(bytes)
        }
    };
}

macro_rules! impl_to {
    ($name:ident, $ty:ty, $to_method:ident, $size:expr) => {
        /// Convert $ty to bytes
        fn $name(val: $ty) -> [u8; $size] {
            val.$to_method()
        }
    };
}

/// Endianness contains implementation for endian converter to/from basic types
/// Default is implemented for Native Endian
pub trait Endianness {
    impl_from!(f64_from_bytes, f64, from_ne_bytes, 8);
    impl_from!(f32_from_bytes, f32, from_ne_bytes, 4);

    impl_from!(u128_from_bytes, u128, from_ne_bytes, 16);
    impl_from!(u64_from_bytes, u64, from_ne_bytes, 8);
    impl_from!(u32_from_bytes, u32, from_ne_bytes, 4);
    impl_from!(u16_from_bytes, u16, from_ne_bytes, 2);
    impl_from!(u8_from_bytes, u8, from_ne_bytes, 1);

    impl_from!(i128_from_bytes, i128, from_ne_bytes, 16);
    impl_from!(i64_from_bytes, i64, from_ne_bytes, 8);
    impl_from!(i32_from_bytes, i32, from_ne_bytes, 4);
    impl_from!(i16_from_bytes, i16, from_ne_bytes, 2);
    impl_from!(i8_from_bytes, i8, from_ne_bytes, 1);

    impl_to!(f64_to_bytes, f64, to_ne_bytes, 8);
    impl_to!(f32_to_bytes, f32, to_ne_bytes, 4);

    impl_to!(u128_to_bytes, u128, to_ne_bytes, 16);
    impl_to!(u64_to_bytes, u64, to_ne_bytes, 8);
    impl_to!(u32_to_bytes, u32, to_ne_bytes, 4);
    impl_to!(u16_to_bytes, u16, to_ne_bytes, 2);
    impl_to!(u8_to_bytes, u8, to_ne_bytes, 1);

    impl_to!(i128_to_bytes, i128, to_ne_bytes, 16);
    impl_to!(i64_to_bytes, i64, to_ne_bytes, 8);
    impl_to!(i32_to_bytes, i32, to_ne_bytes, 4);
    impl_to!(i16_to_bytes, i16, to_ne_bytes, 2);
    impl_to!(i8_to_bytes, i8, to_ne_bytes, 1);
}

/// Native Endian basic types converter
pub struct NativeEndian;
impl Endianness for NativeEndian {}

/// Littme Endian basic types converter
pub struct LittleEndian;
impl Endianness for LittleEndian {
    impl_from!(f64_from_bytes, f64, from_le_bytes, 8);
    impl_from!(f32_from_bytes, f32, from_le_bytes, 4);

    impl_from!(u128_from_bytes, u128, from_le_bytes, 16);
    impl_from!(u64_from_bytes, u64, from_le_bytes, 8);
    impl_from!(u32_from_bytes, u32, from_le_bytes, 4);
    impl_from!(u16_from_bytes, u16, from_le_bytes, 2);
    impl_from!(u8_from_bytes, u8, from_le_bytes, 1);

    impl_from!(i128_from_bytes, i128, from_le_bytes, 16);
    impl_from!(i64_from_bytes, i64, from_le_bytes, 8);
    impl_from!(i32_from_bytes, i32, from_le_bytes, 4);
    impl_from!(i16_from_bytes, i16, from_le_bytes, 2);
    impl_from!(i8_from_bytes, i8, from_le_bytes, 1);

    impl_to!(f64_to_bytes, f64, to_le_bytes, 8);
    impl_to!(f32_to_bytes, f32, to_le_bytes, 4);

    impl_to!(u128_to_bytes, u128, to_le_bytes, 16);
    impl_to!(u64_to_bytes, u64, to_le_bytes, 8);
    impl_to!(u32_to_bytes, u32, to_le_bytes, 4);
    impl_to!(u16_to_bytes, u16, to_le_bytes, 2);
    impl_to!(u8_to_bytes, u8, to_le_bytes, 1);

    impl_to!(i128_to_bytes, i128, to_le_bytes, 16);
    impl_to!(i64_to_bytes, i64, to_le_bytes, 8);
    impl_to!(i32_to_bytes, i32, to_le_bytes, 4);
    impl_to!(i16_to_bytes, i16, to_le_bytes, 2);
    impl_to!(i8_to_bytes, i8, to_le_bytes, 1);
}

/// Big Endian basic types converter
pub struct BigEndian;
impl Endianness for BigEndian {
    impl_from!(f64_from_bytes, f64, from_be_bytes, 8);
    impl_from!(f32_from_bytes, f32, from_be_bytes, 4);

    impl_from!(u128_from_bytes, u128, from_be_bytes, 16);
    impl_from!(u64_from_bytes, u64, from_be_bytes, 8);
    impl_from!(u32_from_bytes, u32, from_be_bytes, 4);
    impl_from!(u16_from_bytes, u16, from_be_bytes, 2);
    impl_from!(u8_from_bytes, u8, from_be_bytes, 1);

    impl_from!(i128_from_bytes, i128, from_be_bytes, 16);
    impl_from!(i64_from_bytes, i64, from_be_bytes, 8);
    impl_from!(i32_from_bytes, i32, from_be_bytes, 4);
    impl_from!(i16_from_bytes, i16, from_be_bytes, 2);
    impl_from!(i8_from_bytes, i8, from_be_bytes, 1);

    impl_to!(f64_to_bytes, f64, to_be_bytes, 8);
    impl_to!(f32_to_bytes, f32, to_be_bytes, 4);

    impl_to!(u128_to_bytes, u128, to_be_bytes, 16);
    impl_to!(u64_to_bytes, u64, to_be_bytes, 8);
    impl_to!(u32_to_bytes, u32, to_be_bytes, 4);
    impl_to!(u16_to_bytes, u16, to_be_bytes, 2);
    impl_to!(u8_to_bytes, u8, to_be_bytes, 1);

    impl_to!(i128_to_bytes, i128, to_be_bytes, 16);
    impl_to!(i64_to_bytes, i64, to_be_bytes, 8);
    impl_to!(i32_to_bytes, i32, to_be_bytes, 4);
    impl_to!(i16_to_bytes, i16, to_be_bytes, 2);
    impl_to!(i8_to_bytes, i8, to_be_bytes, 1);
}
