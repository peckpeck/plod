use binary_rw::{BinaryReader, BinaryWriter};
pub use binary_rw::BinaryError;

type Result<T> = std::result::Result<T,BinaryError>;


pub trait Plod: Sized {
    /// Size on disk (including tag if any)
    fn size(&self) -> usize;
    fn read_from(from: &mut BinaryReader) -> Result<Self>;
    fn write_to(&self, to: &mut BinaryWriter) -> Result<()>;
}

pub use plod_derive::Plod;

#[cfg(test)]
mod tests {
    use super::*;
    use binary_rw::{Endian, MemoryStream, SeekStream};
    use std::fmt::Debug;
    use crate as plod; // we need to know our own name

    #[derive(Plod,PartialEq,Debug)]
    #[plod(tag_type(u8))]
    enum TestEnum1 {
        #[plod(tag=1)]
        A{ x: u8, y: i16 },
        #[plod(tag=2, size_type(u32), byte_sized)]
        B{ x: u8, val: Vec<i16> }
    }
    #[derive(Plod,PartialEq,Debug)]
    #[plod(tag_type(u8))]
    enum TestEnum2 {
        #[plod(tag=1)]
        A(TestStruct1),
        #[plod(tag=2)]
        B(),
        #[plod(tag=3)]
        C,
        #[plod(tag=4,size_type(u16))]
        D(Vec<TestEnum1>),
        #[plod(keep_tag)]
        E(u8,u8),
    }

    #[derive(Plod,PartialEq,Debug)]
    struct TestStruct1 {
        a: u16,
        #[plod(size_type(u32))]
        b: Vec<u8>,
        c: u32,
    }

    #[derive(Plod,PartialEq,Debug)]
    struct TestStruct2 (u16, TestEnum1);

    #[derive(Plod,PartialEq,Debug)]
    struct TestStruct3 {
        #[plod(size_type(u16))]
        a: Vec<u8>,
        b: TestStruct2,
        c: TestEnum2,
        d: TestEnum2,
        e: TestEnum2,
        f: TestEnum2,
        g: TestEnum2,
    }

    #[test]
    fn test_structs() {
        let a1 = TestEnum1::A { x: 1, y: -1 };
        let a1s = 1 + 1 + 2;
        assert_eq!(a1.size(), a1s, "a1");
        it_reads_what_it_writes(&a1);

        let b1 = TestEnum1::B { x: 1, val: vec![1,-1,3] };
        let b1s = 1 + 1 + 4 + 2*3;
        assert_eq!(b1.size(), b1s, "b1");
        it_reads_what_it_writes(&b1);

        let s1 = TestStruct1 {
            a: 1,
            b: vec![1,2,3],
            c: 5,
        };
        let s1s = 2 + 4 + 3 + 4;
        assert_eq!(s1.size(), s1s, "s1");
        it_reads_what_it_writes(&s1);

        let a2 = TestEnum2::A(s1);
        let a2s = 1 + s1s;
        assert_eq!(a2.size(), a2s, "a2");
        it_reads_what_it_writes(&a2);

        let b2 = TestEnum2::B();
        let b2s = 1;
        assert_eq!(b2.size(), b2s, "b2");
        it_reads_what_it_writes(&b2);

        let c2 = TestEnum2::C;
        let c2s = 1;
        assert_eq!(c2.size(), c2s, "c2");
        it_reads_what_it_writes(&c2);

        let d2 = TestEnum2::D(vec![a1]);
        let d2s = 1 + 2 + a1s;
        assert_eq!(d2.size(), d2s, "d2");
        it_reads_what_it_writes(&d2);

        let e2 = TestEnum2::E(5,2);
        let e2s = 1 + 1;
        assert_eq!(e2.size(), e2s, "e2");
        it_reads_what_it_writes(&e2);

        let s2 = TestStruct2(1234, b1);
        let s2s = 2+ b1s;
        assert_eq!(s2.size(), s2s, "s2");
        it_reads_what_it_writes(&s2);

        let s3 = TestStruct3 {
            a: vec![9,8,7,6],
            b: s2,
            c: a2,
            d: b2,
            e: c2,
            f: d2,
            g: e2,
        };
        let s3s = 2+4 + s2s + a2s + b2s + c2s + d2s + e2s;
        assert_eq!(s3.size(), s3s, "s3");
        it_reads_what_it_writes(&s3);
    }

    fn it_reads_what_it_writes<T: Plod+PartialEq+Debug>(t: &T) {
        let mut rw = MemoryStream::new();
        let mut write = BinaryWriter::new(&mut rw, Endian::Big);

        assert!(t.write_to(&mut write).is_ok());

        rw.seek(0).expect("cannot seek");
        let mut read = BinaryReader::new(&mut rw, Endian::Big);
        let result = <T as Plod>::read_from(&mut read);
        //println!("data {:?}", <MemoryStream as Into<Vec<u8>>>::into(rw));
        assert!(result.is_ok());
        assert_eq!(t, &result.unwrap());
    }
}
