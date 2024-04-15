//! macros
//!

use crate::endian::Endianness;
use crate::Plod;
use crate::Result;

use std::io::{Read, Write};

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

/// Unit has no content
impl<E: Endianness> Plod<E> for () {
    // this is endianness independant
    fn size(&self) -> usize {
        0
    }
    fn read_from<R: Read>(_from: &mut R) -> Result<Self> {
        Ok(())
    }
    fn write_to<W: Write>(&self, _to: &mut W) -> Result<()> {
        Ok(())
    }
}

macro_rules! impl_tuples {
    ($($ty:ident, $id:tt), +) => {
        impl<E: Endianness, $($ty: Plod<E>), +> Plod<E> for ($($ty),+) {
            fn size(&self) -> usize { $(self.$id.size() +)  +  0}
            fn read_from<R: Read>(from: &mut R) -> Result<Self> {
                Ok(( $($ty::read_from(from)?), + ))
            }
            fn write_to<W: Write>(&self, to: &mut W) -> Result<()> {
                $(self.$id.write_to(to)?;) +
                Ok(())
            }
        }
    }
}

impl_tuples!( T1, 0, T2, 1 );
impl_tuples!( T1, 0, T2, 1, T3, 2 );
impl_tuples!( T1, 0, T2, 1, T3, 2, T4, 3 );
impl_tuples!( T1, 0, T2, 1, T3, 2, T4, 3, T5, 4 );
impl_tuples!( T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5 );
impl_tuples!( T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6 );
impl_tuples!( T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7 );
impl_tuples!( T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8 );

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
