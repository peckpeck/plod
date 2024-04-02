use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type, Attribute, ExprLit, PathArguments, GenericArgument};
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

struct Attributes {
    /// type of the tag to detect enum variant (per enum)
    tag_type: Option<Ident>,
    /// value of the tag to detect enum variant (per variant)
    tag: Option<ExprLit>,
    /// does this variant retains the tag in its first item
    keep_tag: bool,
    /// is the above retained different from the tag (how much less)
    keep_diff: Option<i64>,
    /// type of the vector size storage
    size_type: Option<Ident>,
    /// is the vector size counted in items or in bytes
    byte_sized: bool,
}

impl Default for Attributes {
    fn default() -> Self {
        Attributes {
            tag_type: None, tag: None,
            keep_tag: false, keep_diff: None,
            size_type: None, byte_sized: false
        }
    }
}

/// Get structure or enum attributes dedicated to this derive
fn get_attributes(attrs: &Vec<Attribute>) -> syn::parse::Result<Attributes> {
    let mut result = Attributes::default();
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
            } else if meta.path.is_ident("byte_sized") {
                result.byte_sized = true;
                Ok(())
            } else if meta.path.is_ident("keep_diff") {
                // TODO
                result.keep_diff = None;
                Ok(())
            } else if meta.path.is_ident("tag_type") {
                meta.parse_nested_meta(|meta| {
                    result.tag_type = meta.path.get_ident().cloned();
                    Ok(())
                })
            } else if meta.path.is_ident("size_type") {
                meta.parse_nested_meta(|meta| {
                    result.size_type = meta.path.get_ident().cloned();
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

fn supported_type(ty: &Ident) -> bool {
    for i in ["bool", "f32", "f64", "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64"] {
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
            if !supported_type(tag_type) {
                return syn::Error::new(tag_type.span(), "plod tag only works with basic types").to_compile_error().into();
            }

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

                // iterate over fields
                let mut size_code = TokenStream::new();
                let mut read_code = TokenStream::new();
                let mut write_code = TokenStream::new();
                let mut field_list = TokenStream::new();
                match &variant.fields {
                    Fields::Named(fields) => {
                        let mut i = 0;
                        for field in fields.named.iter() {
                            // all named fields have an ident
                            let field_ident = field.ident.as_ref().unwrap();
                            unwrap!(generate_for(
                                &field_ident,
                                &field.ty,
                                i == 0 && variant_attributes.keep_tag,
                                &variant_attributes,
                                &mut size_code,
                                &mut read_code,
                                &mut write_code));
                            field_list.extend(quote! {
                                #field_ident,
                            });
                            i += 1;
                        }
                        field_list = quote! { { #field_list } };
                    }
                    Fields::Unnamed(fields) => {
                        for (i,field) in fields.unnamed.iter().enumerate() {
                            let field_ident = Ident::new(&format!("field_{}",i), field.span());
                            unwrap!(generate_for(
                                &field_ident,
                                &field.ty,
                                i == 0 && variant_attributes.keep_tag,
                                &variant_attributes,
                                &mut size_code,
                                &mut read_code,
                                &mut write_code));
                            field_list.extend(quote! {
                                #field_ident,
                            });
                        }
                        field_list = quote! { (#field_list) };
                    }
                    Fields::Unit => {
                        // read code specific
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
                    Self::#ident #field_list => #size_code,
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

fn generate_for(field_ident: &Ident,
                field_type: &Type,
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

                // read code
                if is_tag {
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
                // size code
                let size = known_size(ty);
                size_code.extend(quote! {
                    #size +
                });
            } else {
                let is_vec = match type_path.path.segments.first() {
                    Some(id) => id.ident == "Vec",
                    None => false,
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
                        None => return Err(syn::Error::new(type_path.span(), "#[plod(size_type(<value>)] is mandatory for Vec<type>"))
                    };
                    let read_size = Ident::new(&format!("read_{}", size_ty), field_ident.span());
                    let write_size = Ident::new(&format!("write_{}", size_ty), field_ident.span());
                    generate_for(
                        &Ident::new("item", field_ident.span()),
                        &ty,
                        false,
                        attributes,
                        &mut size_sub,
                        &mut read_sub,
                        &mut write_sub)?;
                    if attributes.byte_sized {
                        size_code.extend(quote! {
                            #field_ident.iter().fold(0, |n, item| n + #size_sub 0) +
                        });
                        read_code.extend(quote! {
                            let mut size = from.#read_size()? as usize;
                            let mut #field_ident = Vec::new();
                            while size > 0 {
                                #read_sub
                                #field_ident.push(item);
                                size -= #size_sub 0;
                            }
                        });
                        write_code.extend(quote! {
                            let size = #field_ident.iter().fold(0, |n, item| n + #size_sub 0);
                            to.#write_size(size as #size_ty)?;
                            for item in #field_ident.iter() {
                                #write_sub
                            }
                        });
                    } else {
                        size_code.extend(quote! {
                            #field_ident.len() +
                        });
                        read_code.extend(quote! {
                            let size = from.#read_size()? as usize;
                            let mut #field_ident = Vec::new();
                            for _ in 0..size {
                                #read_sub
                                #field_ident.push(item);
                            }
                        });
                        write_code.extend(quote! {
                            to.#write_size(#field_ident.len() as #size_ty)?;
                            for item in #field_ident.iter() {
                                #write_sub
                            }
                        });
                    }
                } else {
                    read_code.extend(quote! {
                        let #field_ident = <#type_path as Plod>::read_from(from)?;
                    });
                    write_code.extend(quote! {
                        <#type_path as Plod>::write_to(&#field_ident, to)?;
                    });
                    size_code.extend(quote! {
                        <#type_path as Plod>::size(&#field_ident) +
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
