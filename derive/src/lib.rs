//! # Plod derive crate
//!
//! The documentation is located in the main `` crate

#![deny(missing_docs)]

use proc_macro2::{Span, Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type, Attribute, PathArguments, GenericArgument, LitInt, Pat, Lit};
use syn::parse::Parse;

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
            None => return syn::Error::new($span.span(), $message).to_compile_error().into(),
        }
    };
}

/// The main derive method
#[proc_macro_derive(Plod, attributes(plod))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // generate everything
    let plod = plod_impl(&input);

    // some things
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Build the output
    let expanded = quote! {
        // The generated impl.
        #[automatically_derived]
        impl #impl_generics plod::Plod for #name #ty_generics #where_clause {
            #plod
        }
    };

    // Hand the output tokens back to the compiler
    proc_macro::TokenStream::from(expanded)
}

/// Attributes that ca be used with derive, all in one structure to make it easier to parse.
#[derive(Clone)]
struct Attributes {
    /// type of the tag to detect enum variant (per enum)
    tag_type: Option<Ident>,
    /// value of the tag to detect enum variant (per variant)
    tag: Option<Pat>,
    /// does this variant retains the tag in its first item
    keep_tag: bool,
    /// is the above retained different from the tag (how much less)
    keep_diff: Option<LitInt>,
    /// type of the vector size storage
    size_type: Option<Ident>,
    /// is the vector size counted in items or in bytes
    byte_sized: bool,
    /// Size is off by one
    size_is_next: bool,
    /// endianness of the struct
    endianness: Ident,
    /// magic type and value for this item
    magic: Option<(Ident,Lit)>,
}

impl Default for Attributes {
    fn default() -> Self {
        Attributes {
            tag_type: None, tag: None,
            keep_tag: false, keep_diff: None,
            size_type: None, byte_sized: false, size_is_next: false,
            endianness: Ident::new("NativeEndian", Span::call_site()),
            magic: None,
        }
    }
}

/// A single Attribute structure makes it easier to write parsing code but give worse error reporting
impl Attributes {
    /// Get structure or enum attributes dedicated to this derive
    fn parse(attrs: &Vec<Attribute>) -> syn::parse::Result<Self> {
        let mut result = Attributes::default();
        result._parse(attrs)?;
        Ok(result)
    }

    // sub method of parse and extend
    fn _parse(&mut self, attrs: &Vec<Attribute>) -> syn::parse::Result<()> {
        for attribute in attrs.iter() {
            if !attribute.path().is_ident("plod") {
                continue;
            }
            let meta_parser = syn::meta::parser(|meta| {
                if meta.path.is_ident("tag") {
                    //let value = ExprLit::parse(meta.value()?)?;
                    let value = Pat::parse_multi(meta.value()?)?;
                    self.tag = Some(value);
                    Ok(())
                } else if meta.path.is_ident("keep_diff") {
                    let lit = LitInt::parse(meta.value()?)?;
                    self.keep_diff = Some(lit);
                    self.keep_tag = true;
                    Ok(())
                } else if meta.path.is_ident("big_endian") {
                    self.endianness = Ident::new("BigEndian", Span::call_site());
                    Ok(())
                } else if meta.path.is_ident("size_is_next") {
                    self.size_is_next = true;
                    Ok(())
                } else if meta.path.is_ident("little_endian") {
                    self.endianness = Ident::new("LittleEndian", Span::call_site());
                    Ok(())
                } else if meta.path.is_ident("native_endian") {
                    self.endianness = Ident::new("NativeEndian", Span::call_site());
                    Ok(())
                } else if meta.path.is_ident("keep_tag") {
                    self.keep_tag = true;
                    Ok(())
                } else if meta.path.is_ident("byte_sized") {
                    self.byte_sized = true;
                    Ok(())
                } else if meta.path.is_ident("magic") {
                    meta.parse_nested_meta(|meta| {
                        let ident = meta.path.get_ident()
                            .ok_or(meta.error("Magic must be of the form #[plod(magic(<type>=<value>))]"))?;
                        let lit = Lit::parse(meta.value()?)?;
                        self.magic = Some((ident.clone(), lit));
                        Ok(())
                    })
                } else if meta.path.is_ident("tag_type") {
                    meta.parse_nested_meta(|meta| {
                        self.tag_type = meta.path.get_ident().cloned();
                        Ok(())
                    })
                } else if meta.path.is_ident("size_type") {
                    meta.parse_nested_meta(|meta| {
                        self.size_type = meta.path.get_ident().cloned();
                        Ok(())
                    })
                } else {
                    Err(meta.error("Unsupported plod value"))
                }
            });
            attribute.parse_args_with(meta_parser)?;
        }
        Ok(())
    }

    /// parse attributes that override existing attributes
    fn extend(&self, attrs: &Vec<Attribute>) -> syn::parse::Result<Self> {
        let mut result = self.clone();
        result._parse(attrs)?;
        Ok(result)
    }
}

fn supported_type(ty: &Ident) -> bool {
    for i in ["bool", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128"] {
        if ty == i {
            return true;
        }
    }
    false
}

fn known_size(ty: &Ident) -> usize {
    match ty.to_string().as_str() {
        "bool" => 1,
        "f32" => 4,
        "f64" => 8,
        "i8" => 1,
        "i16" => 2,
        "i32" => 4,
        "i64" => 8,
        "i128" => 16,
        "u8" => 1,
        "u16" => 2,
        "u32" => 4,
        "u64" => 8,
        "u128" => 16,
        _ => panic!("Type must be checked before getting its size"),
    }
}

/// Generate implementation for a given type (struct or enum)
fn plod_impl(input: &DeriveInput) -> TokenStream {
    // get attributes
    let attributes = unwrap!(Attributes::parse(&input.attrs));

    let mut size_impl = TokenStream::new();
    let mut read_impl = TokenStream::new();
    let mut write_impl = TokenStream::new();

    match &input.data {
        Data::Struct(data) => {
            // generate for all fields
            let (size_code, read_code, write_code, field_list) =
                unwrap!(generate_for_fields(&data.fields, Some(&quote!{ self. }), input.ident.span(), &attributes));
            size_impl = size_code;
            read_impl = quote!{
                #read_code
                Ok(Self #field_list)
            };
            write_impl = quote!{
                #write_code
                Ok(())
            };
        }
        Data::Enum(data) => {
            // check enum attributes
            let tag_type = unwrap!(&attributes.tag_type, input.ident, "#[plod(tag_type(<type>)] is mandatory for enum");
            if !supported_type(tag_type) {
                return syn::Error::new(tag_type.span(), "plod tag only works with basic types").to_compile_error().into();
            }

            let read_tag = Ident::new(&format!("read_{}",tag_type), input.ident.span());
            let write_tag = Ident::new(&format!("write_{}",tag_type), input.ident.span());

            // iterate over variants
            let mut default_done = false;
            for variant in data.variants.iter() {
                // check variant attributes
                let variant_attributes = unwrap!(attributes.extend(&variant.attrs));
                let tag_value = &variant_attributes.tag;

                // handle default value
                if default_done {
                    return syn::Error::new(variant.ident.span(), "The variant without #[plod(tag(<value>))] must come last").to_compile_error().into();
                }

                // generate for all fields
                let (size_code, read_code, write_code, field_list) =
                    unwrap!(generate_for_fields(&variant.fields, None, variant.ident.span(), &variant_attributes));

                // code for reading variant
                let ident = &variant.ident;
                match &tag_value {
                    Some(value) =>
                        read_impl.extend(quote! {
                            #value => {
                                #read_code
                                Ok(Self::#ident #field_list)
                            }
                        }),
                    None => {
                        read_impl.extend(quote! {
                            _ => {
                                #read_code
                                Ok(Self::#ident #field_list)
                            }
                        });
                        default_done = true;
                    }
                }

                // code for writing variant
                let add_tag = if variant_attributes.keep_tag {
                    TokenStream::new()
                } else {
                    let tag_pattern = unwrap!(&variant_attributes.tag, ident, "#[plod(tag(<value>))] is mandatory without keep_tag");
                    let tag_value = match tag_pattern {
                        Pat::Lit(expr) => expr,
                        _ =>return syn::Error::new(tag_type.span(), "#[plod(keep_tag)] is mandatory with tag patterns").to_compile_error().into(),
                    };
                    quote!{
                        Self::Endianness::#write_tag(to, #tag_value)?;
                    }
                };
                write_impl.extend(quote!{
                    Self::#ident #field_list => {
                        #add_tag
                        #write_code
                    }
                });

                // code for getting size
                size_impl.extend(quote! {
                    Self::#ident #field_list => #size_code,
                });
            }
            // finalize read_impl
            if default_done {
                read_impl = quote! {
                    let discriminant = Self::Endianness::#read_tag(from)?;
                    match discriminant {
                        #read_impl
                    }
                };
            } else {
                read_impl = quote! {
                    let discriminant = Self::Endianness::#read_tag(from)?;
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
            // Finalize size_impl
            size_impl = quote! {
                match self {
                    #size_impl
                }
            };
        },
        Data::Union(_) => {
            unimplemented!("union")
        },
    }

    let endianness = attributes.endianness;
    quote!{
        type Endianness = plod::#endianness;

        fn size(&self) -> usize {
            #size_impl
        }

        fn read_from<R: std::io::Read>(from: &mut R) -> plod::Result<Self> {
            #read_impl
        }

        fn write_to<W: std::io::Write>(&self, to: &mut W) -> plod::Result<()> {
            #write_impl
        }
    }
}

/// generate code for all fields of a struct / enum variant
fn generate_for_fields(fields: &Fields,
                       field_prefix: Option<&TokenStream>,
                       span: Span,
                       attributes: &Attributes) -> syn::parse::Result<(TokenStream, TokenStream, TokenStream, TokenStream)> {
    let mut size_code = TokenStream::new();
    let mut read_code = TokenStream::new();
    let mut write_code = TokenStream::new();
    let mut field_list = TokenStream::new();
    if let Some((ty, value)) = &attributes.magic {
        let size = known_size(ty);
        let read_tag_i = Ident::new(&format!("read_{}", ty), fields.span());
        let write_tag_i = Ident::new(&format!("write_{}", ty), fields.span());
        // size code
        size_code.extend(quote! {
            #size +
        });
        read_code.extend(quote! {
            let magic = Self::Endianness::#read_tag_i(from)?;
            if magic != #value {
                return Err(std::io::Error::other(format!("Magic value {} expected, found {}", #value, magic)));
            }
        });
        write_code.extend(quote! {
            Self::Endianness::#write_tag_i(to, #value)?;
        });
    }
    match fields {
        Fields::Named(fields) => {
            let mut i = 0;
            for field in fields.named.iter() {
                let field_attributes = attributes.extend(&field.attrs)?;
                // all named fields have an ident
                let field_ident = field.ident.as_ref().unwrap();
                let prefixed_field = match field_prefix {
                    None => field_ident.to_token_stream(),
                    Some(prefix) => quote!{ #prefix #field_ident },
                };
                generate_for_item(
                    &field_ident,
                    &field.ty,
                    &prefixed_field,
                    // TODO field_attributes keep tag ?
                    i == 0 && attributes.keep_tag,
                    &field_attributes,
                    &mut size_code,
                    &mut read_code,
                    &mut write_code)?;
                field_list.extend(quote! {
                    #field_ident,
                });
                i += 1;
            }
            field_list = quote! { { #field_list } };
        }
        Fields::Unnamed(fields) => {
            for (i,field) in fields.unnamed.iter().enumerate() {
                let field_attributes = attributes.extend(&field.attrs)?;
                let field_ident = Ident::new(&format!("field_{}",i), field.span());
                let prefixed_field = match field_prefix {
                    None => field_ident.to_token_stream(),
                    Some(prefix) => {
                        let i = syn::Index::from(i);
                        quote!{ #prefix #i }
                    },
                };
                generate_for_item(
                    &field_ident,
                    &field.ty,
                    &prefixed_field,
                    i == 0 && attributes.keep_tag,
                    &field_attributes,
                    &mut size_code,
                    &mut read_code,
                    &mut write_code)?;
                field_list.extend(quote! {
                    #field_ident,
                });
            }
            field_list = quote! { (#field_list) };
        }
        Fields::Unit => {
            // read code specific
            if attributes.keep_tag {
                return Err(syn::Error::new(span, "Cannot keep tag on unit variant"));
            }
        }
    };
    // final part of size fo the tag
    if attributes.keep_tag {
        size_code.extend(quote!{ 0 });
    } else {
        match &attributes.tag_type {
            None => size_code.extend(quote!{ 0 }),
            Some(ty) => {
                let size = known_size(ty);
                size_code.extend(quote! { #size });
            }
        }
    }
    Ok((size_code, read_code, write_code, field_list))
}

/// Generate code for a single item of a variant or a struct
fn generate_for_item(field_ident: &Ident,
                     field_type: &Type,
                     prefixed_field: &TokenStream,
                     is_tag: bool,
                     attributes: &Attributes,
                     size_code: &mut TokenStream,
                     read_code: &mut TokenStream,
                     write_code: &mut TokenStream) -> syn::parse::Result<()> {
    match field_type {
        Type::Path(type_path) => {
            let supported = match type_path.path.get_ident() {
                Some(ty) => supported_type(ty),
                None => false,
            };
            if supported {
                let ty = type_path.path.get_ident().unwrap();
                let read_tag_i = Ident::new(&format!("read_{}", ty), field_ident.span());
                let write_tag_i = Ident::new(&format!("write_{}", ty), field_ident.span());

                // read and write code
                if is_tag {
                    if let Some(diff) = &attributes.keep_diff {
                        read_code.extend(quote! {
                            let #field_ident = discriminant as #ty - #diff;
                        });
                        write_code.extend(quote! {
                            Self::Endianness::#write_tag_i(to, #prefixed_field + #diff)?;
                        });
                    } else {
                        read_code.extend(quote! {
                            let #field_ident = discriminant as #ty;
                        });
                        write_code.extend(quote! {
                            Self::Endianness::#write_tag_i(to, #prefixed_field)?;
                        });
                    }
                } else {
                    read_code.extend(quote! {
                        let #field_ident = Self::Endianness::#read_tag_i(from)?;
                    });
                    write_code.extend(quote! {
                        Self::Endianness::#write_tag_i(to, #prefixed_field)?;
                    });
                }
                // size code
                let size = known_size(ty);
                size_code.extend(quote! {
                    #size +
                });
            } else {
                let mut is_vec= false;
                let mut is_option= false;
                if let Some(id) = type_path.path.segments.first() {
                    is_vec = id.ident == "Vec";
                    is_option = id.ident == "Option";
                };
                if is_vec {
                    if type_path.path.segments.len() != 1 {
                        return Err(syn::Error::new(type_path.span(), "Only simple Vec supported"));
                    }
                    let args = &type_path.path.segments.first().unwrap().arguments;
                    let angle_args = match args {
                        PathArguments::AngleBracketed(args) => args,
                        _ => return Err(syn::Error::new(type_path.span(), "Only Vec<type> supported")),
                    };
                    if angle_args.args.len() != 1 {
                        return Err(syn::Error::new(type_path.span(), "Only Vec of single type supported"));
                    }
                    let ty = match angle_args.args.first().unwrap() {
                        GenericArgument::Type(Type::Path(ty)) => Type::Path(ty.clone()),
                        _ => return Err(syn::Error::new(type_path.span(), "Only Vec<type> allowed")),
                    };
                    let mut size_sub = TokenStream::new();
                    let mut read_sub = TokenStream::new();
                    let mut write_sub = TokenStream::new();
                    let size_ty = match &attributes.size_type {
                        Some(ty) => ty,
                        None => return Err(syn::Error::new(type_path.span(), "#[plod(size_type(<value>))] is mandatory for Vec<type>"))
                    };
                    let read_size = Ident::new(&format!("read_{}", size_ty), field_ident.span());
                    let write_size = Ident::new(&format!("write_{}", size_ty), field_ident.span());
                    let size_ty_size = known_size(size_ty);
                    let item_ident = Ident::new("item", field_ident.span());
                    generate_for_item(
                        &item_ident,
                        &ty,
                        &item_ident.to_token_stream(),
                        false,
                        attributes,
                        &mut size_sub,
                        &mut read_sub,
                        &mut write_sub)?;
                    size_code.extend(quote! {
                        #size_ty_size + #prefixed_field.iter().fold(0, |n, item| n + #size_sub 0) +
                    });
                    let (plus_one, minus_one) = if attributes.size_is_next {
                        (quote! { + 1 }, quote!{ - 1 })
                    } else {
                        (quote! { }, quote!{ })
                    };
                    if attributes.byte_sized {
                        read_code.extend(quote! {
                            let mut size = Self::Endianness::#read_size(from)? as usize #minus_one;
                            let mut #field_ident = Vec::new();
                            while size > 0 {
                                #read_sub
                                size -= #size_sub 0;
                                #field_ident.push(item);
                            }
                        });
                        write_code.extend(quote! {
                            let size = #prefixed_field.iter().fold(0, |n, item| n + #size_sub 0);
                            Self::Endianness::#write_size(to, size as #size_ty #plus_one)?;
                            for item in #prefixed_field.iter() {
                                #write_sub
                            }
                        });
                    } else {
                        read_code.extend(quote! {
                            let size = Self::Endianness::#read_size(from)? as usize #minus_one;
                            let mut #field_ident = Vec::new();
                            for _ in 0..size {
                                #read_sub
                                #field_ident.push(item);
                            }
                        });
                        write_code.extend(quote! {
                            Self::Endianness::#write_size(to, #prefixed_field.len() as #size_ty #plus_one)?;
                            for item in #prefixed_field.iter() {
                                #write_sub
                            }
                        });
                    }
                } else if is_option {
                    // Options are not written and read as None
                    // so no write code, no size code
                    read_code.extend(quote! {
                        let #field_ident = None;
                    });
                } else {
                    read_code.extend(quote! {
                        let #field_ident = <#type_path as Plod>::read_from(from)?;
                    });
                    write_code.extend(quote! {
                        <#type_path as Plod>::write_to(&#prefixed_field, to)?;
                    });
                    size_code.extend(quote! {
                        <#type_path as Plod>::size(&#prefixed_field) +
                    });
                }
            }
        },
        _ => {
            return Err(syn::Error::new(field_ident.span(), "Unsupported type"));
        },
    }
    Ok(())
}
