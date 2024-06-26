//! # Plod derive implementation
//!
//! Companion crate of plod
//!
//! The documentation is located in the main `plod` crate

#![deny(missing_docs)]

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::Result;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DataEnum, DeriveInput, Fields, GenericArgument, Pat, PathArguments,
    Type, TypePath,
};

use proc_macro2::Span;
use syn::LitInt;

mod attributes;
use attributes::{Attributes, Endianness};

/// produces a token stream of error to warn the final user of the error
macro_rules! unwrap {
    ($expression:expr) => {
        match $expression {
            Ok(a) => a,
            Err(e) => return e.to_compile_error().into(),
        }
    };
    ($expression:expr, $span:expr, $message:literal) => {
        match $expression {
            Some(a) => a,
            None => {
                return syn::Error::new($span.span(), $message)
                    .to_compile_error()
                    .into()
            }
        }
    };
}

/// In some places, only those primitives types are allowed (namely tag and size storage)
fn primitive_type(ty: &Ident) -> bool {
    [
        "f32", "f64", "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128",
    ]
    .iter()
    .any(|i| ty == i)
}

/// We could use `core::mem::size_of` but this is more readable when debugging generated code
fn primitive_size(ty: &Ident) -> LitInt {
    [
        ("f32", 4),
        ("f64", 8),
        ("i8", 1),
        ("i16", 2),
        ("i32", 4),
        ("i64", 8),
        ("i128", 16),
        ("u8", 1),
        ("u16", 2),
        ("u32", 4),
        ("u64", 8),
        ("u128", 16),
    ]
    .iter()
    .find_map(|(i, j)| {
        if ty == i {
            Some(LitInt::new(&j.to_string(), Span::call_site()))
        } else {
            None
        }
    })
    .unwrap()
}

/// Create the proper primitive reader/write method
fn primitive_function(endianness: Endianness) -> (Ident, Ident) {
    let en = match endianness {
        Endianness::Big => "be",
        Endianness::Little => "le",
        Endianness::Native => "ne",
    };
    (
        Ident::new(&format!("from_{}_bytes", en), Span::call_site()),
        Ident::new(&format!("to_{}_bytes", en), Span::call_site()),
    )
}

fn syn_error<S: Spanned, T>(span: &S, message: &str) -> Result<T> {
    Err(syn::Error::new(span.span(), message))
}

/// The main derive method, plod derive is based on obvious plain old data mapping plus some
/// options provided with `#[plod(..)]` attributes.
///
/// Attributes can be inherited, which means that if you define a `#[plod(size_type(u8))]` attribute
/// on a struct, all `Vec` inside this struct will have their size stored as a `u8`;
///
/// Per type attributes:
/// - `#[plod(<endianness>)]` (default: `native_endian`), available values: `native_endian`,
///   `big_endian`, `little_endian`.
/// - `#[plod(<context_type>)]` (default: `()`): the associated type to use when reading and writing data.
///   A context can help when reading and writing data structures.
/// - `#[plod(no_pos)]` (default: `false`): do no generate position handling code used for alignment
/// and padding, it makes slightly shorter code but padding in inner types won't work.
///
/// Enum specific attributes:
/// - `#[plod(tag_type(<tag_type>))]` defines the type used to store the enum discriminant. This must be a
///   primitive type like `u16`, and is stored as the first item of the binary format.
/// - `#[plod(skip)]` (default false), the field will be skipped on serialization, but it must implement `Default`
///   on deserialization.
///q
/// Variant specific attributes:
/// - `#[plod(tag=<tag_value>)]` (implies `keep_tag`, see below) defines a value of type `<tag_type>` used
///   to differentiate each variant. This value can be a match arm (instead of a single value).
/// - `#[plod(keep_tag)]` means that the first field of this variant is used to retain the values
///   that was used as a discriminant. It will be equal to `<tag_value>` if a simple value was
///   provided.
/// - `#[plod(keep_diff=<integer>)]` (implies `keep_tag`) means that the tag also conveys a value,
///   the value is stored after subtracting `<integer>` from the tag. This is especially useful in
///   combination with a tag value that is a range. Eg: `#[plod(tag=6..=8, keep_diff=6)]` will
///   store a value between 0 and 2 included in the first field of this variant when a value
///   between 6 and 8 is encountered during the read.
/// - `#[plod(skip)]` the variant is ignored, it is not created and produces an error of kind Other
///   if encountered during write
///
/// Field item specific attributes:
/// - `#[plod(magic(<type>=<value>))]` the field will be prefixed by a magic value. This value must be present
///   at rest. It is written with `write_to` and its presence is checked by `read_from` but not stored.
/// - `#[plod(skip)]` (default: false), the field will be skipped on serialization, but it must implement `Default`
///   to be created on deserialization.
/// - `#[plod(is_context)]` (default: false): this field will be used as the context for all next fields
///   encountered in this structure.
///
/// Vec field specific attributes:
/// - `#[plod(size_type(<size_type>))]` defines the type used to store the `Vec` size. This must
///   be an integer type. The default is to store the number of items as the _size_.
/// - `#[plod(bytes_sized)]` means that the size stored is the number of bytes instead of the numer
///   of items in the `Vec`
/// - `#[plod(size_is_next)]` means that the bytes used to store the `Vec` size contains the place
///   for the next entry instead of the length of the vector ie: n+1
///
#[proc_macro_derive(Plod, attributes(plod))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // get main attributes
    let attributes = unwrap!(Attributes::parse(&input.attrs));

    // generate everything
    let plod_impl = unwrap!(plod_impl(&input, &attributes));

    // thing for generation
    let name = input.ident;
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();
    let type_params = input.generics.type_params();

    // define endianness generic
    let ctx_ty = attributes.context_type;

    // Build the output
    let expanded = quote! {
        // The generated impl.
        #[automatically_derived]
        impl <#(#type_params),*> plod::Plod for #name #ty_generics #where_clause {
            type Context= #ctx_ty;
            #plod_impl
        }
    };

    // Hand the output tokens back to the compiler
    proc_macro::TokenStream::from(expanded)
}

/// Generate implementation for a given input type (struct or enum)
fn plod_impl(input: &DeriveInput, attributes: &Attributes) -> Result<TokenStream> {
    let self_name = &input.ident;

    let (size_impl, read_impl, write_impl) = match &input.data {
        Data::Struct(data) => {
            // generate for all fields
            let (size_code, read_code, write_code, field_list) = generate_for_fields(
                &data.fields,
                Some(&quote! { self. }),
                &input.ident,
                &attributes,
            )?;
            (
                size_code,
                quote! {
                    #read_code
                    Ok(#self_name #field_list)
                },
                quote! {
                    #write_code
                    Ok(())
                },
            )
        }
        Data::Enum(data) => enum_impl(self_name, data, attributes)?,
        Data::Union(u) => {
            return Err(syn::Error::new(
                u.union_token.span(),
                "Union types are not supported by plod",
            ))
        }
    };

    Ok(quote! {
        fn size_at_rest(&self) -> usize {
            #size_impl
        }

        fn impl_read_from<R: std::io::Read>(from: &mut R, ctx: &Self::Context, mut _pos: usize) -> plod::Result<Self> {
            #read_impl
        }

        fn impl_write_to<W: std::io::Write>(&self, to: &mut W, ctx: &Self::Context, mut _pos: usize) -> plod::Result<()> {
            #write_impl
        }
    })
}

/// Generate code for all variants of an enum
fn enum_impl(
    self_name: &Ident,
    data: &DataEnum,
    attributes: &Attributes,
) -> Result<(TokenStream, TokenStream, TokenStream)> {
    let mut size_impl = TokenStream::new();
    let mut read_impl = TokenStream::new();
    let mut write_impl = TokenStream::new();

    // _Note_: It's the Enum that reads the discriminant, but it's the variant that writes
    //   the discriminant. This is because we need it for the read match, but we may not know
    //   the exact value before reading the variant.

    // check enum attributes
    let tag_type = match &attributes.tag_type {
        Some(t) => t,
        None => return syn_error(self_name, "#[plod(tag_type(<type>)] is mandatory for enum"),
    };
    if !primitive_type(tag_type) {
        return syn_error(
            &tag_type,
            "#[plod(tag_type(<type>)] tag only works with primitive types",
        );
    }
    let tag_size = primitive_size(tag_type);
    let (from_method, to_method) = primitive_function(attributes.endianness);

    // iterate over variants
    let mut default_done = false;
    for variant in data.variants.iter() {
        let ident = &variant.ident;

        // check variant attributes
        let variant_attributes = attributes.extend(&variant.attrs)?;
        let tag_value = &variant_attributes.tag;

        // handle skipped values, no size code, no read code, error on write
        if variant_attributes.skip {
            let error_token = quote! { #self_name::#ident };
            let error_str = error_token.to_string();
            let fields_token = if let Fields::Unit = variant.fields {
                TokenStream::new()
            } else {
                quote! { (..) }
            };
            size_impl.extend(quote! {
                #self_name::#ident #fields_token => 0,
            });
            write_impl.extend(quote! {
                #self_name::#ident #fields_token => {
                    return Err(std::io::Error::other(format!("Variant {} cannot be written  because it is plod(skipped)", #error_str)));
                }
            });
            continue;
        }

        // handle default value
        if default_done {
            return syn_error(
                &variant.ident,
                "The variant without #[plod(tag(<value>))] must come last",
            );
        }

        // generate for all fields
        let (size_code, read_code, write_code, field_list) =
            generate_for_fields(&variant.fields, None, &variant.ident, &variant_attributes)?;

        // code for reading variant
        match &tag_value {
            Some(value) => read_impl.extend(quote! {
                #value => {
                    #read_code
                    Ok(#self_name::#ident #field_list)
                }
            }),
            None => {
                read_impl.extend(quote! {
                    _ => {
                        #read_code
                        Ok(#self_name::#ident #field_list)
                    }
                });
                default_done = true;
            }
        }

        // code for writing variant
        let add_tag = if variant_attributes.keep_tag {
            TokenStream::new()
        } else {
            let tag_pattern = match &variant_attributes.tag {
                Some(t) => t,
                None => {
                    return syn_error(ident, "#[plod(tag(<value>))] is mandatory without keep_tag")
                }
            };
            let tag_value = match tag_pattern {
                Pat::Lit(expr) => expr,
                _ => {
                    return syn_error(tag_type, "#[plod(keep_tag)] is mandatory with tag patterns")
                }
            };
            quote! {
                let buffer: [u8; #tag_size] = (#tag_value as #tag_type).#to_method();
                to.write_all(&buffer)?;
            }
        };
        write_impl.extend(quote! {
            #self_name::#ident #field_list => {
                #add_tag
                #write_code
            }
        });

        // code for getting size
        size_impl.extend(quote! {
            #self_name::#ident #field_list => #size_code,
        });
    }
    // Finalize size_impl
    size_impl = quote! {
        match self {
            #size_impl
        }
    };
    // finalize read_impl
    let read_tag = quote! {
        let mut buffer: [u8; #tag_size] = [0; #tag_size];
        from.read_exact(&mut buffer)?;
        let discriminant = #tag_type::#from_method(buffer);
        _pos += #tag_size;
    };
    if default_done {
        read_impl = quote! {
            #read_tag
            match discriminant {
                #read_impl
            }
        };
    } else {
        read_impl = quote! {
            #read_tag
            match discriminant {
                #read_impl
                _ => return Err(std::io::Error::other(format!("Tag value {} not found", discriminant))),
            }
        };
    }
    // Finalize write_impl
    write_impl = quote! {
        match self {
            #write_impl
        }
        Ok(())
    };
    Ok((size_impl, read_impl, write_impl))
}

/// generate code for all fields of a struct / enum variant
fn generate_for_fields(
    fields: &Fields,
    field_prefix: Option<&TokenStream>,
    ident: &Ident,
    attributes: &Attributes,
) -> Result<(TokenStream, TokenStream, TokenStream, TokenStream)> {
    let mut size_code = TokenStream::new();
    let mut read_code = TokenStream::new();
    let mut write_code = TokenStream::new();
    let mut field_list = TokenStream::new();
    let mut context_val = quote! { ctx };
    let mut prefixed_context_val = quote! { ctx };
    if let Some((ty, value)) = &attributes.magic {
        let (from_method, to_method) = primitive_function(attributes.endianness);
        if !primitive_type(ty) {
            return syn_error(ty, "magic only works with primitive types");
        }
        let ty_size = primitive_size(ty);

        // size code
        size_code.extend(quote! {
            #ty_size +
        });
        read_code.extend(quote! {
            let mut buffer: [u8; #ty_size] = [0; #ty_size];
            from.read_exact(&mut buffer)?;
            let magic = #ty::#from_method(buffer);
            if magic != #value {
                return Err(std::io::Error::other(format!("Magic value {} expected, found {}", #value, magic)));
            }
            _pos += #ty_size;
        });
        write_code.extend(quote! {
            let buffer: [u8; #ty_size] = (#value as #ty).#to_method();
            to.write_all(&buffer)?;
            _pos += #ty_size;
        });
    }
    match fields {
        Fields::Named(fields) => {
            let mut i = 0;
            for field in fields.named.iter() {
                let field_attributes = attributes.extend(&field.attrs)?;
                // all named fields have an ident
                let field_ident = field.ident.as_ref().unwrap();
                let (prefixed_field_ref, prefixed_field_dotted) = match field_prefix {
                    None => (quote! { #field_ident }, quote! { #field_ident .}),
                    Some(prefix) => (
                        quote! {  (& #prefix #field_ident) },
                        quote! {  #prefix #field_ident . },
                    ),
                };
                generate_for_item(
                    &field_ident,
                    &field.ty,
                    &prefixed_field_ref,
                    &prefixed_field_dotted,
                    // TODO field_attributes keep tag ?
                    i == 0 && attributes.keep_tag,
                    &field_attributes,
                    &mut size_code,
                    &mut read_code,
                    &mut write_code,
                    &context_val,
                    &prefixed_context_val,
                )?;
                if field_attributes.is_context {
                    context_val = quote! { (&#field_ident) };
                    prefixed_context_val = prefixed_field_ref;
                }
                field_list.extend(quote! {
                    #field_ident,
                });
                i += 1;
            }
            field_list = quote! { { #field_list } };
        }
        Fields::Unnamed(fields) => {
            for (i, field) in fields.unnamed.iter().enumerate() {
                let field_attributes = attributes.extend(&field.attrs)?;
                let field_ident = Ident::new(&format!("field_{}", i), field.span());
                let (prefixed_field_ref, prefixed_field_dotted) = match field_prefix {
                    None => (quote! { #field_ident }, quote! { #field_ident .}),
                    Some(prefix) => {
                        let i = syn::Index::from(i);
                        (quote! {  ( & #prefix #i ) }, quote! {  #prefix #i . })
                    }
                };
                generate_for_item(
                    &field_ident,
                    &field.ty,
                    &prefixed_field_ref,
                    &prefixed_field_dotted,
                    i == 0 && attributes.keep_tag,
                    &field_attributes,
                    &mut size_code,
                    &mut read_code,
                    &mut write_code,
                    &context_val,
                    &prefixed_context_val,
                )?;
                if field_attributes.is_context {
                    context_val = quote! { (&#field_ident) };
                    prefixed_context_val = quote! { #prefixed_field_ref };
                }
                field_list.extend(quote! {
                    #field_ident,
                });
            }
            field_list = quote! { (#field_list) };
        }
        Fields::Unit => {
            // read code specific
            if attributes.keep_tag {
                return syn_error(ident, "Cannot keep tag on unit variant");
            }
        }
    };
    // final part of size for the tag
    if attributes.keep_tag {
        size_code.extend(quote! { 0 });
    } else {
        match &attributes.tag_type {
            None => size_code.extend(quote! { 0 }),
            Some(ty) => {
                let ty_size = primitive_size(ty);
                size_code.extend(quote! { #ty_size });
            }
        }
    }
    Ok((size_code, read_code, write_code, field_list))
}

/// Generate code for a single item of a variant or a struct
fn generate_for_item(
    field_ident: &Ident,
    field_type: &Type,
    prefixed_field_ref: &TokenStream,
    prefixed_field_dotted: &TokenStream,
    is_tag: bool,
    attributes: &Attributes,
    size_code: &mut TokenStream,
    read_code: &mut TokenStream,
    write_code: &mut TokenStream,
    context_val: &TokenStream,
    prefixed_context_val: &TokenStream,
) -> Result<()> {
    if attributes.skip {
        // no size code, no write code
        // default on read
        read_code.extend(quote! {
            let #field_ident = <#field_type as std::default::Default>::default();
        });
        return Ok(());
    }
    match field_type {
        Type::Path(type_path) => {
            let mut is_vec = false;
            let mut is_primitive = false;
            if let Some(id) = type_path.path.segments.first() {
                is_vec = id.ident == "Vec";
                // TODO we should probably make sure there is only one segment
                is_primitive = primitive_type(&id.ident);
            };
            if is_vec {
                generate_for_vec(
                    type_path,
                    field_ident,
                    prefixed_field_dotted,
                    attributes,
                    size_code,
                    read_code,
                    write_code,
                    context_val,
                    prefixed_context_val,
                )?;
            } else if is_primitive {
                let ty = type_path.path.get_ident().unwrap();
                let ty_size = primitive_size(ty);
                let (from_method, to_method) = primitive_function(attributes.endianness);
                size_code.extend(quote! {
                    #ty_size +
                });
                if is_tag {
                    // TODO, tag should always be read/written by enum_impl, this would be easier
                    if let Some(diff) = &attributes.keep_diff {
                        read_code.extend(quote! {
                            let #field_ident = discriminant as #ty - #diff;
                        });
                    } else {
                        read_code.extend(quote! {
                            let #field_ident = discriminant as #ty;
                        });
                    }
                } else {
                    read_code.extend(quote! {
                        let mut buffer: [u8; #ty_size] = [0; #ty_size];
                        from.read_exact(&mut buffer)?;
                        let #field_ident = #ty::#from_method(buffer);
                        _pos += #ty_size;
                    });
                }
                let diff = if is_tag && attributes.keep_diff.is_some() {
                    let diff = attributes.keep_diff.as_ref().unwrap();
                    quote! { + #diff }
                } else {
                    TokenStream::new()
                };
                write_code.extend(quote! {
                    let buffer: [u8; #ty_size] = (#prefixed_field_ref #diff). #to_method();
                    to.write_all(&buffer)?;
                    _pos += #ty_size;
                });
            } else {
                size_code.extend(quote! {
                    <#type_path as plod::Plod>::size_at_rest(#prefixed_field_ref) +
                });
                read_code.extend(quote! {
                    let #field_ident = <#type_path as plod::Plod>::impl_read_from(from, #context_val.into(), _pos)?;
                    _pos += <#type_path as plod::Plod>::size_at_rest(&#field_ident);
                });
                write_code.extend(quote! {
                    <#type_path as plod::Plod>::impl_write_to(#prefixed_field_ref, to, #prefixed_context_val.into(), _pos)?;
                    _pos += <#type_path as plod::Plod>::size_at_rest(#prefixed_field_ref);
                });
            }
        }
        Type::Tuple(t) => {
            let mut field_list = TokenStream::new();
            for (i, field_ty) in t.elems.iter().enumerate() {
                let field_ident = Ident::new(&format!("infield_{}", i), field_ty.span());
                let (prefixed_field_ref, prefixed_field_dotted) = {
                    let i = syn::Index::from(i);
                    (
                        quote! {  ( & #prefixed_field_dotted #i ) },
                        quote! {  #prefixed_field_dotted #i . },
                    )
                };
                generate_for_item(
                    &field_ident,
                    field_ty,
                    &prefixed_field_ref,
                    &prefixed_field_dotted,
                    false,
                    attributes,
                    size_code,
                    read_code,
                    write_code,
                    context_val,
                    prefixed_context_val,
                )?;
                field_list.extend(quote! {
                    #field_ident,
                });
            }
            read_code.extend(quote! {
                let #field_ident = (#field_list);
            });
        }
        Type::Array(t) => {
            let n = &t.len;
            let ty_ = &t.elem;
            // u8 special case
            let mut vec_u8 = false;
            if let Type::Path(type_path) = ty_.as_ref() {
                if let Some(id) = type_path.path.segments.first() {
                    vec_u8 = id.ident == "u8";
                }
            }

            if vec_u8 {
                size_code.extend(quote! {
                    #prefixed_field_dotted len() +
                });
                read_code.extend(quote! {
                    let mut #field_ident: #t = [0; #n];
                    from.read_exact(&mut #field_ident)?;
                });
                write_code.extend(quote! {
                    to.write_all(#prefixed_field_dotted as_slice())?;
                });
            } else {
                let mut item_size_code = TokenStream::new();
                let mut item_read_code = TokenStream::new();
                let mut item_write_code = TokenStream::new();
                let item_name = Ident::new("item", field_ident.span());
                generate_for_item(
                    &item_name,
                    ty_,
                    &quote! { #item_name },
                    &quote! { #item_name . },
                    false,
                    attributes,
                    &mut item_size_code,
                    &mut item_read_code,
                    &mut item_write_code,
                    context_val,
                    prefixed_context_val,
                )?;
                size_code.extend(quote! {
                    #prefixed_field_dotted iter().fold(0, |n, item| n + #item_size_code 0) +
                });
                read_code.extend(quote! {
                    let mut vec = Vec::new();
                    for _ in 0..#n {
                        #item_read_code
                        vec.push(item);
                    }
                    let #field_ident: #t = vec.try_into().unwrap();
               });
                write_code.extend(quote! {
                    for item in #prefixed_field_dotted iter() {
                        #item_write_code
                    }
                });
            }
        }
        _ => {
            return syn_error(field_ident, "Unsupported type for Plod");
        }
    }
    Ok(())
}

fn generate_for_vec(
    type_path: &TypePath,
    field_ident: &Ident,
    prefixed_field_dotted: &TokenStream,
    attributes: &Attributes,
    size_code: &mut TokenStream,
    read_code: &mut TokenStream,
    write_code: &mut TokenStream,
    context_val: &TokenStream,
    prefixed_context_val: &     TokenStream,
) -> Result<()> {
    let size_ty = match &attributes.size_type {
        Some(ty) => ty,
        None => {
            return syn_error(
                type_path,
                "#[plod(size_type(<value>))] is mandatory for Vec<type>",
            );
        }
    };
    if !primitive_type(size_ty) {
        return syn_error(size_ty, "vec length magic only works with primitive types");
    }
    let ty_size = primitive_size(size_ty);

    let (from_method, to_method) = primitive_function(attributes.endianness);
    // we can unwrap because it's how we know we are in a vec
    let vec_generic = match &type_path.path.segments.first().unwrap().arguments {
        PathArguments::AngleBracketed(pa) => {
            if pa.args.len() != 1 {
                return syn_error(
                    type_path,
                    "Plod only support regular Vec<Type>: unknown type Vec<X,Y,...>",
                );
            }
            match pa.args.first().unwrap() {
                GenericArgument::Type(t) => t,
                _ => {
                    return syn_error(
                        type_path,
                        "Plod only support regular Vec<Type>: unknown Vec<...>",
                    )
                }
            }
        }
        _ => {
            return syn_error(
                type_path,
                "Plod only support regular Vec<Type>: unknown Vec...",
            );
        }
    };
    // u8 special case
    let mut vec_u8 = false;
    if let Type::Path(type_path) = vec_generic {
        if let Some(id) = type_path.path.segments.first() {
            vec_u8 = id.ident == "u8";
        }
    }

    let mut item_size_code = TokenStream::new();
    let mut item_read_code = TokenStream::new();
    let mut item_write_code = TokenStream::new();
    let item_name = Ident::new("item", field_ident.span());
    let it_name = Ident::new("it", field_ident.span());

    if vec_u8 {
        size_code.extend(quote! {
            #ty_size + #prefixed_field_dotted len() +
        });
    } else {
        generate_for_item(
            &item_name,
            vec_generic,
            &quote! { # it_name },
            &quote! { #it_name . },
            false,
            attributes,
            &mut item_size_code,
            &mut item_read_code,
            &mut item_write_code,
            context_val,
            prefixed_context_val,
        )?;

        // it_name may or may not be used by item_size_code
        size_code.extend(quote! {
            #ty_size + #prefixed_field_dotted iter().fold(0, #[allow(unused_variables)] |n, #it_name| n + #item_size_code 0) +
        });
    }
    let (plus_one, minus_one) = if attributes.size_is_next {
        (quote! { + 1 }, quote! { - 1 })
    } else {
        (quote! {}, quote! {})
    };
    read_code.extend(quote! {
        let mut buffer: [u8; #ty_size] = [0; #ty_size];
        from.read_exact(&mut buffer)?;
        _pos += #ty_size;
        let mut size = #size_ty::#from_method(buffer) as usize #minus_one;
    });
    if attributes.byte_sized {
        write_code.extend(quote! {
            let size = #prefixed_field_dotted iter().fold(0, #[allow(unused_variables)] |n, #it_name| n + #item_size_code 0);
            let buffer: [u8; #ty_size] = (size as #size_ty #plus_one).#to_method();
            to.write_all(&buffer)?;
            _pos += #ty_size;
        });
    } else {
        write_code.extend(quote! {
            let size = #prefixed_field_dotted len();
            let buffer: [u8; #ty_size] = (size as #size_ty #plus_one).#to_method();
            to.write_all(&buffer)?;
            _pos += #ty_size;
        });
    }
    // Vec<u8> can be read and written all at once
    if vec_u8 {
        // byte size == count size for Vec<u8>
        read_code.extend(quote! {
            let mut #field_ident = vec![0_u8; size];
            from.read_exact(&mut #field_ident)?;
            _pos += size;
        });
        write_code.extend(quote! {
            to.write_all(#prefixed_field_dotted as_slice())?;
            _pos += size;
        });
    } else {
        if attributes.byte_sized {
            read_code.extend(quote! {
                let mut #field_ident = Vec::new();
                while size > 0 {
                    #item_read_code
                    let #it_name = &#item_name;
                    size -= #item_size_code 0;
                    #field_ident.push(item);
                }
            });
        } else {
            read_code.extend(quote! {
                let mut #field_ident = Vec::new();
                for _ in 0..size {
                    #item_read_code
                    #field_ident.push(#item_name);
                }
            });
        }
        write_code.extend(quote! {
            for #it_name in #prefixed_field_dotted iter() {
                #item_write_code
            }
        });
    }
    Ok(())
}
