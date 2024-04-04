use crate::error::Result;
use std::io::{Read, Write};

macro_rules! impl_read {
    ($name:ident, $ty:ty, $from_method:ident, $size:expr) => {
        fn $name(&mut self) -> Result<$ty> {
            let mut buffer: [u8; $size] = [0; $size];
            self.read_exact(&mut buffer)?;
            Ok(<$ty>::$from_method(buffer))
        }
    };
    (read_bool) => {
        fn read_bool(&mut self) -> Result<bool> {
            let mut buffer = [0_u8];
            self.read_exact(&mut buffer)?;
            Ok(buffer[0] > 0)
        }
    };
    (read_bytes) => {
        fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>> {
            let mut buffer: Vec<u8> = vec![0; length];
            self.read_exact(&mut buffer)?;
            Ok(buffer)
        }
    };
}

macro_rules! impl_write {
    ($name:ident, $ty:ty, $to_method:ident, $size:expr) => {
        fn $name(&mut self, val: $ty) -> Result<()> {
            let buffer: [u8; $size] = val.$to_method();
            Ok(self.write_all(&buffer)?)
        }
    };
    (write_bool) => {
        fn write_bool(&mut self, b: bool) -> Result<()> {
            let buffer = [ if b { 1_u8 } else { 0_u8 } ];
            Ok(self.write_all(&buffer)?)
        }
    };
    (write_bytes) => {
        fn write_bytes(&mut self, val: &[u8]) -> Result<()> {
            Ok(self.write_all(val)?)
        }
    };
}

pub struct BigEndian;
pub struct LittleEndian;
pub struct NativeEndian;

pub trait Endianness {}
impl Endianness for BigEndian {}
impl Endianness for LittleEndian {}
impl Endianness for NativeEndian {}

pub trait EndianRead<E: Endianness> {
    fn read_f64(&mut self) -> Result<f64>;
    fn read_f32(&mut self) -> Result<f32>;
    fn read_u64(&mut self) -> Result<u64>;
    fn read_u32(&mut self) -> Result<u32>;
    fn read_u16(&mut self) -> Result<u16>;
    fn read_u8(&mut self)  -> Result<u8>;
    fn read_i64(&mut self) -> Result<i64>;
    fn read_i32(&mut self) -> Result<i32>;
    fn read_i16(&mut self) -> Result<i16>;
    fn read_i8(&mut self)  -> Result<i8>;
    fn read_bool(&mut self)  -> Result<bool>;
    fn read_bytes(&mut self, length: usize)  -> Result<Vec<u8>>;
}

pub trait EndianWrite<E: Endianness> {
    fn write_f64(&mut self, val: f64) -> Result<()>;
    fn write_f32(&mut self, val: f32) -> Result<()>;
    fn write_u64(&mut self, val: u64) -> Result<()>;
    fn write_u32(&mut self, val: u32) -> Result<()>;
    fn write_u16(&mut self, val: u16) -> Result<()>;
    fn write_u8(&mut self,  val: u8)  -> Result<()>;
    fn write_i64(&mut self, val: i64) -> Result<()>;
    fn write_i32(&mut self, val: i32) -> Result<()>;
    fn write_i16(&mut self, val: i16) -> Result<()>;
    fn write_i8(&mut self,  val: i8)  -> Result<()>;
    fn write_bool(&mut self, val: bool) -> Result<()>;
    fn write_bytes(&mut self, val: &[u8]) -> Result<()>;
}

/// Big Endian reader.
impl<T: Read> EndianRead<BigEndian> for T {
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
}

/// Little Endian reader.
impl<T: Read> EndianRead<LittleEndian> for T {
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
}

/// Native Endian reader.
impl<T: Read> EndianRead<NativeEndian> for T {
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
}

/// Big Endian Writer.
impl<T: Write> EndianWrite<BigEndian> for T {
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

/// Little Endian Writer.
impl<T: Write> EndianWrite<LittleEndian> for T {
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

/// Native Endian Writer.
impl<T: Write> EndianWrite<NativeEndian> for T {
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

