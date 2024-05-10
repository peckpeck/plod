#![deny(missing_docs)]

/// Endianness contains implementation for endian converter to/from basic types
/// Default is implemented for Native Endian
pub trait Endianness {}

/// Native Endian basic types converter
pub struct NativeEndian;
impl Endianness for NativeEndian {}

/// Little Endian basic types converter
pub struct LittleEndian;
impl Endianness for LittleEndian {}

/// Big Endian basic types converter
pub struct BigEndian;
impl Endianness for BigEndian {}
