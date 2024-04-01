use binary_rw::{BinaryError, BinaryReader, BinaryWriter};

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
enum MyStruct {
    #[plod(tag=1)]
    A(u8),
    #[plod(keep_tag)]
    B(u8,u8),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as plod; // we need to know our own name

    #[derive(Plod)]
    #[plod(tag_type(u8))]
    enum MyStruct {
        #[plod(tag=1)]
        A(u8),
        #[plod(tag=2)]
        B(u8,u8),
    }
    /*    //#[pos(BigEndian)]
        #[derive(PosReadWrite)]
        struct MyStruct {
            a: u16,
            //#[pod(len=usize)]
            b: Vec<u8>,
            csize: u32,
            c: C,
        }

        //#[pos(size=provided)]
        struct C {
            data: Vec<u8>,
        }*/
}
