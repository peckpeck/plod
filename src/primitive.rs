#![deny(missing_docs)]

use crate::Plod;
use crate::Result;
use crate::endian::Endianness;

use std::io::{Read, Write};

macro_rules! impl_plod {
    ($ty:ty, $from_method:ident, $to_method:ident, $size:expr) => {
        impl<E: Endianness> Plod<E> for $ty {
            fn size(&self) -> usize { $size }

            fn read_from<R: Read>(from: &mut R) -> Result<Self> {
                let mut buffer: [u8; $size] = [0; $size];
                from.read_exact(&mut buffer)?;
                Ok(E::$from_method(buffer))
            }

            fn write_to<W: Write>(&self, to: &mut W) -> Result<()> {
                let buffer: [u8; $size] = E::$to_method(*self);
                Ok(to.write_all(&buffer)?)
            }
        }
    };
}

impl_plod!(f64, f64_from_bytes, f64_to_bytes, 8);
impl_plod!(f32, f32_from_bytes, f32_to_bytes, 4);

impl_plod!(u128, u128_from_bytes, u128_to_bytes, 16);
impl_plod!(u64,  u64_from_bytes,  u64_to_bytes,  8);
impl_plod!(u32,  u32_from_bytes,  u32_to_bytes,  4);
impl_plod!(u16,  u16_from_bytes,  u16_to_bytes,  2);
impl_plod!(u8,   u8_from_bytes,   u8_to_bytes,   1);

impl_plod!(i128, i128_from_bytes, i128_to_bytes, 16);
impl_plod!(i64,  i64_from_bytes,  i64_to_bytes,  8);
impl_plod!(i32,  i32_from_bytes,  i32_to_bytes,  4);
impl_plod!(i16,  i16_from_bytes,  i16_to_bytes,  2);
impl_plod!(i8,   i8_from_bytes,   i8_to_bytes,   1);

// manual implementation for Endian independant types
impl<E: Endianness> Plod<E> for bool {
    fn size(&self) -> usize { 1 }
    fn read_from<R: Read>(from: &mut R) -> Result<Self> {
        let b = <u8 as Plod<E>>::read_from(from)?;
        Ok(b>0)
    }
    fn write_to<W: Write>(&self, to: &mut W) -> Result<()> {
        let b = if *self { 1_u8 } else { 0_u8 };
        <u8 as Plod<E>>::write_to(&b, to)
    }
}