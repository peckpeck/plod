# plod

## plod, deriving plain old data

Plod is an easy to use plain old data reader and writer.
It transforms them from and to natural rust types.

Plain old are were commonly designed to be used in C, but in rust we can have more meaningful
datastructures for teh same representation. For example, in C unions with a separate tag are
the only way to represent the thing called enum that we have for grated in rust.

Since it uses the standard `Read` and `Write` traits, Plod can be used to read and write
binary files as well as network protocols as long as you have a reader or a writer.

Here is an example with a struct and an enum:
```rust
use plod::Plod;

#[derive(Plod)]
struct Value {
    kind: u16,
    value: u32,
}

#[derive(Plod)]
#[plod(tag_type(u8))]
enum ValueStore{
    #[plod(tag=1)]
    Series {
        kind: u16,
        #[plod(size_type(u16))]
        values: Vec<u32> },
    #[plod(tag=2,size_type(u16))]
    Keys(Vec<Value>),
}
```

If you want to serialize your own data to a common format, you may prefer Serde

If your file format is not binary you may prefer ...

If your data is a pure struct with only primary types you may prefer pod or ...



why use plod ?
 ...
no interaction with #repr
 ...
comparison with plain, pod, bytemuck
 ...

Example, tutorial, first use

How derive is handled :
- enum
- struct
- Vec
- Option

Attributes (explicit all defaults) :
- ...

Document endianness and it inheritance

How to call Plod trait methods

Return io::Error::Other on unexpected tag

License: MIT
