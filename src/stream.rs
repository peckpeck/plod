use std::io::{Read, Write};
use std::borrow::Borrow;
use super::Result;

macro_rules! impl_read {
    ($name:ident, $ty:ty, $from_method:ident, $size:expr) => {
        pub fn $name<R: Read>(read: &mut R) -> Result<$ty> {
            let mut buffer: [u8; $size] = [0; $size];
            read.read_exact(&mut buffer)?;
            Ok(<$ty>::$from_method(buffer))
        }
    };
    (read_bool) => {
        pub fn read_bool<R: Read>(read: &mut R) -> Result<bool> {
            let mut buffer = [0_u8];
            read.read_exact(&mut buffer)?;
            Ok(buffer[0] > 0)
        }
    };
    (read_bytes) => {
        pub fn read_bytes<R: Read>(read: &mut R, length: usize) -> Result<Vec<u8>> {
            let mut buffer: Vec<u8> = vec![0; length];
            read.read_exact(&mut buffer)?;
            Ok(buffer)
        }
    };
}

macro_rules! impl_write {
    ($name:ident, $ty:ty, $to_method:ident, $size:expr) => {
        pub fn $name<V: Borrow<$ty>, W: Write>(write: &mut W, val: V) -> Result<()> {
            let buffer: [u8; $size] = val.borrow().$to_method();
            Ok(write.write_all(&buffer)?)
        }
    };
    (write_bool) => {
        pub fn write_bool<B: Borrow<bool>, W: Write>(write: &mut W, b: B) -> Result<()> {
            let buffer = [ if *b.borrow() { 1_u8 } else { 0_u8 } ];
            Ok(write.write_all(&buffer)?)
        }
    };
    (write_bytes) => {
        pub fn write_bytes<W: Write>(write: &mut W, val: &[u8]) -> Result<()> {
            Ok(write.write_all(val)?)
        }
    };
}

/// Big Endian reader and writer
pub struct BigEndian;
impl BigEndian {
    impl_read!(read_f64, f64, from_be_bytes, 8);
    impl_read!(read_f32, f32, from_be_bytes, 4);

    impl_read!(read_u64, u64, from_be_bytes, 8);
    impl_read!(read_u32, u32, from_be_bytes, 4);
    impl_read!(read_u16, u16, from_be_bytes, 2);
    impl_read!(read_u8,   u8, from_be_bytes, 1);

    impl_read!(read_i64, i64, from_be_bytes, 8);
    impl_read!(read_i32, i32, from_be_bytes, 4);
    impl_read!(read_i16, i16, from_be_bytes, 2);
    impl_read!(read_i8,   i8, from_be_bytes, 1);

    impl_read!(read_bool);
    impl_read!(read_bytes);

    impl_write!(write_f64, f64, to_be_bytes, 8);
    impl_write!(write_f32, f32, to_be_bytes, 4);

    impl_write!(write_u64, u64, to_be_bytes, 8);
    impl_write!(write_u32, u32, to_be_bytes, 4);
    impl_write!(write_u16, u16, to_be_bytes, 2);
    impl_write!(write_u8,   u8, to_be_bytes, 1);

    impl_write!(write_i64, i64, to_be_bytes, 8);
    impl_write!(write_i32, i32, to_be_bytes, 4);
    impl_write!(write_i16, i16, to_be_bytes, 2);
    impl_write!(write_i8,   i8, to_be_bytes, 1);

    impl_write!(write_bool);
    impl_write!(write_bytes);
}

/// Little Endian reader and writer
pub struct LittleEndian;
impl LittleEndian {
    impl_read!(read_f64, f64, from_le_bytes, 8);
    impl_read!(read_f32, f32, from_le_bytes, 4);

    impl_read!(read_u64, u64, from_le_bytes, 8);
    impl_read!(read_u32, u32, from_le_bytes, 4);
    impl_read!(read_u16, u16, from_le_bytes, 2);
    impl_read!(read_u8,   u8, from_le_bytes, 1);

    impl_read!(read_i64, i64, from_le_bytes, 8);
    impl_read!(read_i32, i32, from_le_bytes, 4);
    impl_read!(read_i16, i16, from_le_bytes, 2);
    impl_read!(read_i8,   i8, from_le_bytes, 1);

    impl_read!(read_bool);
    impl_read!(read_bytes);

    impl_write!(write_f64, f64, to_le_bytes, 8);
    impl_write!(write_f32, f32, to_le_bytes, 4);

    impl_write!(write_u64, u64, to_le_bytes, 8);
    impl_write!(write_u32, u32, to_le_bytes, 4);
    impl_write!(write_u16, u16, to_le_bytes, 2);
    impl_write!(write_u8,   u8, to_le_bytes, 1);

    impl_write!(write_i64, i64, to_le_bytes, 8);
    impl_write!(write_i32, i32, to_le_bytes, 4);
    impl_write!(write_i16, i16, to_le_bytes, 2);
    impl_write!(write_i8,   i8, to_le_bytes, 1);

    impl_write!(write_bool);
    impl_write!(write_bytes);
}

/// Native Endian reader and writer
pub struct NativeEndian;
impl NativeEndian {
    impl_read!(read_f64, f64, from_ne_bytes, 8);
    impl_read!(read_f32, f32, from_ne_bytes, 4);

    impl_read!(read_u64, u64, from_ne_bytes, 8);
    impl_read!(read_u32, u32, from_ne_bytes, 4);
    impl_read!(read_u16, u16, from_ne_bytes, 2);
    impl_read!(read_u8,   u8, from_ne_bytes, 1);

    impl_read!(read_i64, i64, from_ne_bytes, 8);
    impl_read!(read_i32, i32, from_ne_bytes, 4);
    impl_read!(read_i16, i16, from_ne_bytes, 2);
    impl_read!(read_i8,   i8, from_ne_bytes, 1);

    impl_read!(read_bool);
    impl_read!(read_bytes);

    impl_write!(write_f64, f64, to_ne_bytes, 8);
    impl_write!(write_f32, f32, to_ne_bytes, 4);

    impl_write!(write_u64, u64, to_ne_bytes, 8);
    impl_write!(write_u32, u32, to_ne_bytes, 4);
    impl_write!(write_u16, u16, to_ne_bytes, 2);
    impl_write!(write_u8,   u8, to_ne_bytes, 1);

    impl_write!(write_i64, i64, to_ne_bytes, 8);
    impl_write!(write_i32, i32, to_ne_bytes, 4);
    impl_write!(write_i16, i16, to_ne_bytes, 2);
    impl_write!(write_i8,   i8, to_ne_bytes, 1);

    impl_write!(write_bool);
    impl_write!(write_bytes);
}

/// To limit
pub trait Endianness {}
impl Endianness for BigEndian {}
impl Endianness for LittleEndian {}
impl Endianness for NativeEndian {}
