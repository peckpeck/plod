#![deny(missing_docs)]

use crate::endian::Endianness;
use crate::Plod;
use crate::Result;

use std::io::{Read, Write};

/// TODO impl for tuples

/// Option is implemented as None, this is used to allow better structures for the user
impl<T, E: Endianness> Plod<E> for Option<T> {
    // this is endianness independant
    fn size(&self) -> usize {
        0
    }
    fn read_from<R: Read>(_from: &mut R) -> Result<Self> {
        Ok(None)
    }
    fn write_to<W: Write>(&self, _to: &mut W) -> Result<()> {
        Ok(())
    }
}

/// Plod implementation helper for generic Vec
pub fn vec_size<E: Endianness, T: Plod<E>>(vec: &[T]) -> usize {
    vec.iter().fold(0, |n, item| n + item.size())
}

/// Plod implementation helper for generic Vec
pub fn vec_read_from_item_count<E: Endianness, T: Plod<E>, R: Read>(
    size: usize,
    from: &mut R,
) -> Result<Vec<T>> {
    let mut vec = Vec::new();
    for _ in 0..size {
        vec.push(T::read_from(from)?);
    }
    Ok(vec)
}

/// Plod implementation helper for generic Vec
pub fn vec_read_from_byte_count<E: Endianness, T: Plod<E>, R: Read>(
    mut size: usize,
    from: &mut R,
) -> Result<Vec<T>> {
    let mut vec = Vec::new();
    while size > 0 {
        let t = T::read_from(from)?;
        size -= t.size();
        vec.push(t);
    }
    Ok(vec)
}

/// Plod implementation helper for generic Vec
pub fn vec_write_to<E: Endianness, T: Plod<E>, W: Write>(vec: &[T], to: &mut W) -> Result<()> {
    for i in vec {
        i.write_to(to)?;
    }
    Ok(())
}
