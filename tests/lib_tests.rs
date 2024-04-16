use plod::*;
use std::fmt::Debug;

#[derive(Plod, PartialEq, Debug)]
#[plod(tag_type(u8))]
enum TestEnum1 {
    #[plod(tag = 1)]
    A { x: u8, y: i16, z: u128 },
    #[plod(tag = 2, size_type(u32), byte_sized)]
    B { x: u8, val: Vec<i16> },
    #[plod(skip)]
    C,
    #[plod(skip)]
    D(u16),
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
    h: bool,
    #[plod(skip)]
    i: i32,
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
        h: true,
        i: 0,
    };
    let s1s = 2 + 2 + 4 + 3 + 4 + (2 + 4) + 3 * 2 + 1;
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
        g: [2, 3, 4],
        h: false,
        i: 10,
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
        g: [2, 3, 4],
        h: false,
        i: 0,
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
    b: Vec<(u16, u16)>,
    #[plod(size_type(u16))]
    c: Vec<T>,
    #[plod(size_type(u16))]
    d: Vec<TestVec<bool>>,
}

#[test]
fn test_vecs() {
    let vec = TestVec {
        a: vec![1],
        b: vec![(2, 3)],
        c: vec![4],
        d: vec![],
    };
    it_reads_what_it_writes(&vec);
}

#[test]
fn test_tuple() {
    let t = (1, 2);
    it_reads_what_it_writes(&t);
}

#[test]
fn test_skip_fail() {
    let s1 = TestEnum1::C;
    let s2 = TestEnum1::D(0);
    let mut memory: Vec<u8> = Vec::new();
    assert!(Plod::write_to(&s1, &mut memory).is_err());
    assert!(Plod::write_to(&s2, &mut memory).is_err());
}

// TODO test with generic in struct
// TODO test endianness mix and match
