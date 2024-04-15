//! # plod, deriving plain old data
//!
//! Plod is an easy to use plain old data reader and writer.
//! It transforms them from and to natural rust types.
//!
//! Plain old are were commonly designed to be used in C, but in rust we can have more meaningful
//! datastructures for teh same representation. For example, in C unions with a separate tag are
//! the only way to represent the thing called enum that we have for grated in rust.
//!
//! Since it uses the standard `Read` and `Write` traits, Plod can be used to read and write
//! binary files as well as network protocols as long as you have a reader or a writer.
//!
//! Here is an example with a struct and an enum
//! ```
//! use plod::Plod;
//!
//! #[derive(Plod)]
//! struct Value {
//!     kind: u16,
//!     value: u32,
//! }
//!
//! #[derive(Plod)]
//! #[plod(tag_type(u8))]
//! enum ValueStore{
//!     #[plod(tag=1)]
//!     Series {
//!         kind: u16,
//!         #[plod(size_type(u16))]
//!         values: Vec<u32> },
//!     #[plod(tag=2,size_type(u16))]
//!     Keys(Vec<Value>),
//! }
//! ```
//!
//! If you want to serialize your own data to a common format, you may prefer Serde
//!
//! If your file format is not binary you may prefer ...
//!
//! If your data is a pure struct with only primary types you may prefer pod or ...
//!
//!
//!
//! why use plod ?
//!  ...
//! no inyteraction with #repr
//!  ...
//! comparision with plain, pod, bytemuck
//!  ...
//!
//! Example, tutorial, first use
//!
//! How derive is handled :
//! - enum
//! - struct
//! - Vec
//! - Option
//!
//! Attributes (explicit all defaults) :
//! - ...
//!
//! Document endianness and it inheritance
//!
//! How to call Plod trait methods
//!
//! Return io::Error::Other on unexpected tag

#![deny(missing_docs)]

mod endian;
mod primitive;
// This only contains derive helper, so hide it
#[doc(hidden)]
pub mod generic;

pub use endian::{BigEndian, Endianness, LittleEndian, NativeEndian};
use std::io::{Read, Write};

/// plod results Result uses io errors
pub type Result<T> = std::result::Result<T, std::io::Error>;

/// The main plain old data trait
/// It is usually implemented using `#[derive(Plod)]`, but it can also be implemented manually to
/// handle specific cases
pub trait Plod<E: Endianness=NativeEndian>: Sized {
    /// Size once serialized (including tag if any)
    fn size(&self) -> usize;

    /// Read this structure from a reader
    fn read_from<R: Read>(from: &mut R) -> Result<Self>;

    /// Write this structure to a writer
    fn write_to<W: Write>(&self, to: &mut W) -> Result<()>;
}

pub use plod_derive::Plod;

#[cfg(test)]
mod tests {
    use super::*;
    use crate as plod;
    use std::fmt::Debug; // we need to know our own name because it is used by derive

    #[derive(Plod, PartialEq, Debug)]
    #[plod(tag_type(u8))]
    enum TestEnum1 {
        #[plod(tag = 1)]
        A { x: u8, y: i16, z: u128 },
        #[plod(tag = 2, size_type(u32), byte_sized)]
        B { x: u8, val: Vec<i16> },
    }

    #[derive(Plod, PartialEq, Debug)]
    #[plod(tag_type(i8))]
    enum TestEnum2 {
        #[plod(tag = 1)]
        A(TestStruct1),
        #[plod(tag = 2)]
        B(),
        #[plod(tag = 3)]
        C,
        #[plod(tag = 4, size_type(u16))]
        D(Vec<TestEnum1>),
        #[plod(tag = 5, keep_tag)]
        E(i8, u8),
        #[plod(tag=6..=8|10, keep_tag)]
        F(i8, u8),
        #[plod(keep_tag, keep_diff=-5)]
        G(i8, u8),
    }

    #[derive(Plod, PartialEq, Debug)]
    #[plod(magic(u16 = 0xbaba))]
    struct TestStruct1 {
        a: u16,
        #[plod(size_type(u32))]
        b: Vec<u8>,
        c: u32,
        d: Option<u32>,
        e: (),
        f: (u16, u32),
        g: [u16; 3],
    }

    #[derive(Plod, PartialEq, Debug)]
    struct TestStruct2(u16, TestEnum1);

    #[derive(Plod, PartialEq, Debug)]
    struct TestStruct3 {
        #[plod(size_type(u16))]
        a: Vec<u8>,
        b: TestStruct2,
        c: TestEnum2,
        d: TestEnum2,
        e: TestEnum2,
        f: TestEnum2,
        g: TestEnum2,
        h: TestEnum2,
        i: TestEnum2,
    }

    #[test]
    fn test_structs() {
        let a1 = TestEnum1::A {
            x: 1,
            y: -1,
            z: u128::MAX,
        };
        let a1s = 1 + 1 + 2 + 16;
        assert_eq!(a1.size(), a1s, "a1");
        it_reads_what_it_writes(&a1);

        let b1 = TestEnum1::B {
            x: 1,
            val: vec![1, -1, 3],
        };
        let b1s = 1 + 1 + 4 + 2 * 3;
        assert_eq!(b1.size(), b1s, "b1");
        it_reads_what_it_writes(&b1);

        let s1 = TestStruct1 {
            a: 1,
            b: vec![1, 2, 3],
            c: 5,
            d: None,
            e: (),
            f: (1, 2),
            g: [1, 2, 3],
        };
        let s1s = 2 + 2 + 4 + 3 + 4 + (2+4) + 3*2;
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

        let e2 = TestEnum2::E(5, 2);
        let e2s = 1 + 1;
        assert_eq!(e2.size(), e2s, "e2");
        it_reads_what_it_writes(&e2);

        let f2 = TestEnum2::F(7, 2);
        let f2s = 1 + 1;
        assert_eq!(f2.size(), f2s, "f2");
        it_reads_what_it_writes(&f2);

        let g2 = TestEnum2::G(14, 2);
        let g2s = 1 + 1;
        assert_eq!(g2.size(), g2s, "g2");
        it_reads_what_it_writes(&g2);

        let s2 = TestStruct2(1234, b1);
        let s2s = 2 + b1s;
        assert_eq!(s2.size(), s2s, "s2");
        it_reads_what_it_writes(&s2);

        let s3 = TestStruct3 {
            a: vec![9, 8, 7, 6],
            b: s2,
            c: a2,
            d: b2,
            e: c2,
            f: d2,
            g: e2,
            h: f2,
            i: g2,
        };
        let s3s = 2 + 4 + s2s + a2s + b2s + c2s + d2s + e2s + f2s + g2s;
        assert_eq!(s3.size(), s3s, "s3");
        it_reads_what_it_writes(&s3);
    }

    fn it_reads_what_it_writes<T: Plod + PartialEq + Debug>(t: &T) {
        let mut memory: Vec<u8> = Vec::new();
        assert!(t.write_to(&mut memory).is_ok());

        let mut mem = std::io::Cursor::new(memory);
        let result = T::read_from(&mut mem);
        //println!("data {:?}", <MemoryStream as Into<Vec<u8>>>::into(rw));
        assert!(result.is_ok(), "read struct error");
        assert_eq!(t, &result.unwrap());
    }

    #[derive(Plod, PartialEq, Debug)]
    #[plod(big_endian, magic(u16 = 0xabcd))]
    struct TestMagic {
        a: u16,
    }

    #[test]
    fn test_magic() {
        let big = TestMagic { a: 0x1234 };
        let mut memory: Vec<u8> = Vec::new();
        assert!(big.write_to(&mut memory).is_ok());
        assert_eq!(memory, vec![0xab, 0xcd, 0x12, 0x34]);
    }

    #[test]
    fn test_option() {
        let s1 = TestStruct1 {
            a: 1,
            b: vec![1, 2, 3],
            c: 5,
            d: Some(45),
            e: (),
            f: (1, 2),
            g: [2,3,4],
        };
        let mut memory: Vec<u8> = Vec::new();
        assert!(s1.write_to(&mut memory).is_ok());

        let mut mem = std::io::Cursor::new(memory);
        let result = TestStruct1::read_from(&mut mem);
        assert!(result.is_ok());

        let s2 = TestStruct1 {
            a: 1,
            b: vec![1, 2, 3],
            c: 5,
            d: None,
            e: (),
            f: (1, 2),
            g: [2,3,4],
        };
        assert_eq!(s2, result.unwrap());
    }

    #[derive(Plod, PartialEq, Debug)]
    #[plod(any_endian)]
    struct TestEndian {
        a: u32,
        #[plod(size_type(u16))]
        b: Vec<u16>,
    }

    #[test]
    fn test_endianness() {
        let big = TestEndian {
            a: 0x12345678,
            b: vec![1, 2],
        };
        let mut memory: Vec<u8> = Vec::new();
        assert!(
            <TestEndian as Plod<BigEndian>>::write_to(&big, &mut memory).is_ok(),
            "write big endian"
        );
        assert_eq!(
            memory,
            vec![0x12, 0x34, 0x56, 0x78, 0x00, 0x02, 0x00, 0x01, 0x00, 0x02],
            "big endian repr"
        );

        let little = TestEndian {
            a: 0x12345678,
            b: vec![2, 3],
        };
        let mut memory: Vec<u8> = Vec::new();
        assert!(
            //<TestEndian as Plod<LittleEndian>>::write_to(&little, &mut memory).is_ok(),
            Plod::<LittleEndian>::write_to(&little, &mut memory).is_ok(),
            "write little endian"
        );
        assert_eq!(
            memory,
            vec![0x78, 0x56, 0x34, 0x12, 0x02, 0x00, 0x02, 0x00, 0x03, 0x00],
            "little endian repr"
        );
    }

    #[derive(Plod, PartialEq, Debug)]
    struct TestVec<T: Plod> {
        #[plod(size_type(u16))]
        a: Vec<u32>,
        #[plod(size_type(u16))]
        b: Vec<(u16,u16)>,
        #[plod(size_type(u16))]
        c: Vec<T>,
        #[plod(size_type(u16))]
        d: Vec<TestVec<bool>>,
    }

    #[test]
    fn test_vecs() {
        let vec = TestVec {
            a: vec![1], b: vec![(2,3)], c: vec![4], d: vec![]
        };
        it_reads_what_it_writes(&vec);
    }

    #[test]
    fn test_tuple() {
        let t = (1,2);
        it_reads_what_it_writes(&t);
    }

    // TODO test with generic in struct
    // TODO test endianness mix and match
}
