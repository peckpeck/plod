use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type, Attribute, ExprLit};
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

struct Attributes {
    tag_type: Option<Ident>,
    tag: Option<ExprLit>,
    keep_tag: bool,
    keep_diff: bool,
}

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

/// Get structure or enum attributes dedicated to this derive
fn get_attributes(attrs: &Vec<Attribute>) -> syn::parse::Result<Attributes> {
    let mut result = Attributes{ tag_type: None, tag: None, keep_tag: false, keep_diff: false };
    for attribute in attrs.iter() {
        if !attribute.path().is_ident("plod") {
            continue;
        }
        let meta_parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("tag") {
                let value = ExprLit::parse(meta.value()?)?;
                result.tag = Some(value);
                Ok(())
            } else if meta.path.is_ident("keep_tag") {
                result.keep_tag = true;
                Ok(())
            } else if meta.path.is_ident("keep_diff") {
                result.keep_diff = true;
                Ok(())
            } else if meta.path.is_ident("tag_type") {
                meta.parse_nested_meta(|meta| {
                    result.tag_type = meta.path.get_ident().cloned();
                    Ok(())
                })
            } else {
                Err(meta.error("Unsupported plod value"))
            }
        });
        attribute.parse_args_with(meta_parser)?;
    }
    Ok(result)
}

fn supported_type(ty: &Ident) -> syn::parse::Result<()> {
    for i in ["bool", "f32", "f64", "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64"] {
        if ty == i {
            return Ok(());
        }
    }
    Err(syn::Error::new(ty.span(), "plod only works with basic types"))
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
        "u8" => 1,
        "u16" => 2,
        "u32" => 4,
        "u64" => 8,
        _ => panic!("Type must be checked before getting its size"),
    }
}

fn plod_impl(input: &DeriveInput) -> TokenStream {
    // get attributes
    let attributes = unwrap!(get_attributes(&input.attrs));

    let mut size_impl = TokenStream::new();
    let mut read_impl = TokenStream::new();
    let mut write_impl = TokenStream::new();

    match &input.data {
        Data::Struct(data) => {
            match data.fields {
                Fields::Named(_) => {}
                Fields::Unnamed(_) => {}
                Fields::Unit => {} // just ignore
            }
            unimplemented!("struct")
        }
        Data::Enum(data) => {
            // check enum attributes
            let tag_type = unwrap!(&attributes.tag_type, input.ident, "#[plod(tag_type(<type>)] is mandatory for enum");
            unwrap!(supported_type(tag_type));

            let read_tag = Ident::new(&format!("read_{}",tag_type), input.ident.span());
            let write_tag = Ident::new(&format!("write_{}",tag_type), input.ident.span());

            // iterate over variants
            let mut default_done = false;
            for variant in data.variants.iter() {
                // check variant attributes
                let variant_attributes = unwrap!(get_attributes(&variant.attrs));
                let tag_value = &variant_attributes.tag;

                // handle default value
                if default_done {
                    return syn::Error::new(input.ident.span(), "The variant without #[plod(tag(<value>)] must come last").to_compile_error().into();
                }

                // iterate over fields1
                let mut size_code = TokenStream::new();
                let mut read_code = TokenStream::new();
                let mut write_code = TokenStream::new();
                let mut field_list = TokenStream::new();
                let mut free_field_list = TokenStream::new();
                match &variant.fields {
                    Fields::Named(fields) => {}
                    Fields::Unnamed(fields) => {
                        for (i,field) in fields.unnamed.iter().enumerate() {
                            let field_ident = Ident::new(&format!("field_{}",i), field.span());
                            match &field.ty {
                                Type::Path(p) => {
                                    let ty = unwrap!(p.path.get_ident(), field.span(), "Unknown type error");
                                    unwrap!(supported_type(ty));

                                    let read_tag_i = Ident::new(&format!("read_{}",ty), field.span());
                                    let write_tag_i = Ident::new(&format!("write_{}",ty), field.span());

                                    // read code
                                    if i == 0 && variant_attributes.keep_tag {
                                        read_code.extend(quote! {
                                            let #field_ident = discriminant;
                                        });
                                    } else {
                                        read_code.extend(quote! {
                                            let #field_ident = from.#read_tag_i()?;
                                        });
                                    }
                                    // Write code
                                    write_code.extend(quote! {
                                        to.#write_tag_i(#field_ident)?;
                                    });
                                    field_list.extend(quote! {
                                        #field_ident,
                                    });
                                    // size code
                                    let size = known_size(ty);
                                    size_code.extend(quote!{
                                        #size +
                                    })
                                },
                                _ => { return syn::Error::new(field.span(), "Unsupported type").to_compile_error(); },
                            }
                        }
                        field_list = quote! { (#field_list) };
                        free_field_list = quote!{ (..) };
                    }
                    Fields::Unit => {
                        // read code
                        if variant_attributes.keep_tag {
                            return syn::Error::new(variant.span(), "Cannot keep tag on unit variant").to_compile_error();
                        }
                    }
                };

                // code for reading variant
                let ident = &variant.ident;
                read_code.extend(quote!{
                    Ok(Self::#ident #field_list)
                });
                match &tag_value {
                    Some(value) =>
                        read_impl.extend(quote! {
                            #value => {
                                #read_code
                            }
                        }),
                    None => {
                        read_impl.extend(quote! {
                            _ => {
                                #read_code
                            }
                        });
                        default_done = true;
                    }
                }

                // code for writing variant
                let add_tag = if variant_attributes.keep_tag {
                    quote!{ }
                } else {
                    let tag_value = unwrap!(&variant_attributes.tag, ident, "#[plod(tag(<value>)] is mandatory without keep_tag");
                    quote!{
                        to.#write_tag(#tag_value)?;
                    }
                };
                write_impl.extend(quote!{
                    Self::#ident #field_list => {
                        #add_tag
                        #write_code
                    }
                });

                // code for getting size
                if variant_attributes.keep_tag {
                    size_code.extend(quote!{ 0 });
                } else {
                    let size = known_size(tag_type);
                    size_code.extend(quote!{ #size });
                };
                size_impl.extend(quote! {
                    Self::#ident #free_field_list => #size_code,
                });
            }
            // finalize read_impl
            if default_done {
                read_impl = quote! {
                    let discriminant = from.#read_tag()?;
                    match discriminant {
                        #read_impl
                    }
                };
            } else {
                read_impl = quote! {
                    let discriminant = from.#read_tag()?;
                    match discriminant {
                        #read_impl
                       // TODO better error
                        _ => return Err(BinaryError::InvalidChar),
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

    quote!{
        fn size(&self) -> usize {
            #size_impl
        }

        fn read_from(from: &mut BinaryReader) -> std::result::Result<Self,plod::BinaryError> {
            #read_impl
        }

        fn write_to(&self, to: &mut BinaryWriter) -> std::result::Result<(),plod::BinaryError> {
            #write_impl
        }
    }
}
