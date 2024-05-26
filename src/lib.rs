//! # plod, deriving plain old data
//!
//! Plod is an easy-to-use plain old data reader and writer.
//! It transforms them from and to natural rust types like `enum` or `Vec`.
//!
//! Plain old are commonly designed to be used with C, but in rust we can have more meaningful
//! datastructures for the same representation. For example, in C, unions with a separate tag are
//! the only way to represent the thing called enum that we have for granted in rust.
//!
//! Since it uses the standard `Read` and `Write` traits, Plod can be used to read and write
//! binary files as well as network protocols as long as you have a reader or a writer.
//! The `Plod` trait also have access to a context and position to allow for more cases where
//! automatic derive is possible.
//!
//! Here is an example with a struct and an enum:
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
//! More documentation about `#[plod]` attributes at [`Plod`](macro@Plod)
//!
//! # Why use plod ?
//!
//! Plod transforms a serialized plain old data into a ready to use plain Rust data-structure.
//! It uses attributes to achieve that.
//! This means that an `enum` or a `Vec` can easily be read from a binary file.
//!
//! Other reasons:
//! * Plod is based on generic `Read`and `Write` traits.
//! * Plod knows about endianness during serialization, it reorders bytes for you.
//! * Plod doesn't use unsafe or transmute for read and write
//! * Plod doesn't need you to play with `#[repr()]`
//!
//! Plod is for *plain old data*, which means that is well suited for known, existing, binary formats.
//! *But*:
//! - If you want a way to serialize *your own* data and be able to read it later, you should
//! prefer serde which can serialize any data into many more formats that can be self describing.
//!
//! - If your file format is *not binary* and not easily supported by serde, you may look at
//! [`nom`] for parsing it.
//!
//! - If your data only contains primary types and all you want is speed you may take a look at
//! [`plain`], [`pod`] and [`nue`].
//!
//! # Special cases
//!
//! Plod use the obvious representation for struct as C does. However, some data structure are not so obvious.
//! - `enum` are represented with a specific tag at the start, each variant can have its own size
//! - `Vec` are represented with their size at the start (either in bytes or in item count)
//! - Skipped values are not represented, they are ignored when written and replaced with `default()`when read.
//!
//! Document endianness and it inheritance
//!
//! How to call Plod trait methods
//!
//! Example, tutorial, first use
//!
#![deny(missing_docs)]

use std::io::{Read, Write};

/// plod results Result uses io errors
pub type Result<T> = std::result::Result<T, std::io::Error>;

/// The main thing
pub use plod_derive::Plod;

/// The main plain old data trait.
///
/// It is usually implemented using `#[derive(Plod)]`, but it can also be implemented manually to
/// handle specific cases.
/// The endianness is implied by implementation and not a generic type because it makes calling and writing
/// the trait much easier and because I never found a data format that is both big and little
/// endian (counter example anyone ?).
pub trait Plod: Sized {
    /// Context passed to read and write methods, if you don't need one, just use `()`
    /// The context will be passed down to any sub method called by `read_from` and `write_to`
    /// It is passed using `into()` on `&Context`. This means that `From<&Context>` should be implemented
    /// for any other context used inside the current data structure. Especially, it implies that you
    /// must `impl  From<&Context> for ()` since all primitive types use `()` as a context.
    type Context;

    /// Size once serialized (including tag if any)
    // also used internally by byte sized Vec
    fn size_at_rest(&self) -> usize;

    /// Read this structure from a reader
    /// Returns `std::io::Error` in case or error
    /// Returns an error of kind `std::io::ErrorKind::Other` if an unknown enum tag was found
    fn read_from<R: Read>(from: &mut R) -> Result<Self>
        where Self::Context : Default
    { Self::impl_read_from(from, &Self::Context::default(), 0) }

    /// Same as `read_from` with all parameters, you must implement this one.
    /// You should call this one if you are reading from a Plod implementation.
    /// `pos` is the position in bytes in the reader, it is used to handle padding and alignment.
    fn impl_read_from<R: Read>(from: &mut R, ctx: &Self::Context, pos: usize) -> Result<Self>;

    /// Write this structure to a writer
    /// Returns `std::io::Error` in case or error
    fn write_to<W: Write>(&self, to: &mut W) -> Result<()>
        where Self::Context : Default
    { self.impl_write_to(to, &mut Self::Context::default(), 0) }

    /// Same as `write_to` with all parameters, you must implement this one.
    /// You should call this one if you are writing from a Plod implementation.
    /// `pos` is the position in bytes in the writer, it is used to handle padding and alignment.
    fn impl_write_to<W: Write>(&self, to: &mut W, ctx: &Self::Context, pos: usize) -> Result<()>;
}

// everything in this library is public and is tested via integration tests
