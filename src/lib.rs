use binary_rw::{BinaryReader, BinaryWriter};
pub use binary_rw::BinaryError;

type Result<T> = std::result::Result<T,BinaryError>;


pub trait Plod: Sized {
    /// Size on disk (including tag if any)
    fn size(&self) -> usize;
    fn read_from(from: &mut BinaryReader) -> Result<Self>;
    /*fn read_sized(from: &mut BinaryReader, _size: usize) -> Result<Self> {
        Self::read_from(from)
    }*/
    fn write_to(&self, to: &mut BinaryWriter) -> Result<()>;
}

pub use plod_derive::Plod;


use crate as plod; // we need to know our own name

#[derive(Plod)]
#[plod(tag_type(u8))]
enum MyX {
    #[plod(tag=1)]
    A{ x: u8, y: i16 },
    #[plod(tag=2, size_type(u32), byte_sized)]
    B{ x: u8, val: Vec<i16> }
}
#[derive(Plod)]
#[plod(tag_type(u8))]
enum MyEnum {
    #[plod(tag=1)]
    A(MyStruct),
    #[plod(tag=2)]
    B(),
    #[plod(tag=3)]
    C,
    #[plod(tag=4,size_type(u16))]
    D(Vec<MyX>),
    #[plod(keep_tag)]
    E(u8,u8),
}

#[derive(Plod)]
struct MyStruct {
    a: u16,
    #[plod(size_type(u32))]
    b: Vec<u8>,
    csize: u32,
    c: C,
}

#[derive(Plod)]
struct C {
    #[plod(size_type(u16))]
    data: Vec<u8>,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate as plod; // we need to know our own name

    #[derive(Plod)]
    #[plod(tag_type(u8))]
    enum MyX {
        #[plod(tag=1)]
        A{ x: u8, y: i16 },
        #[plod(tag=2, size_type(u32), byte_sized)]
        B{ x: u8, val: Vec<i16> }
    }
    #[derive(Plod)]
    #[plod(tag_type(u8))]
    enum MyEnum {
        #[plod(tag=1)]
        A(MyStruct),
        #[plod(tag=2)]
        B(),
        #[plod(tag=3)]
        C,
        #[plod(tag=4,size_type(u16))]
        D(Vec<MyX>),
        #[plod(keep_tag)]
        E(u8,u8),
    }

    #[derive(Plod)]
    struct MyStruct {
        a: u16,
        #[plod(size_type(u32))]
        b: Vec<u8>,
        csize: u32,
        c: C,
    }

    #[derive(Plod)]
    struct C {
        #[plod(size_type(u16))]
        data: Vec<u8>,
    }
}
