use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::{Parse, Result};
use syn::{Pat, Type, Lit, LitInt, Attribute};

/// Attributes that ca be used with derive, all in one structure to make it easier to parse.
#[derive(Clone)]
pub struct Attributes {
    /// type of the tag to detect enum variant (per enum)
    pub tag_type: Option<Ident>,
    /// value of the tag to detect enum variant (per variant)
    pub tag: Option<Pat>,
    /// does this variant retains the tag in its first item
    pub keep_tag: bool,
    /// is the above retained different from the tag (how much less)
    pub keep_diff: Option<LitInt>,
    /// type of the vector size storage
    pub size_type: Option<Ident>,
    /// is the vector size counted in items or in bytes
    pub byte_sized: bool,
    /// Size is off by one
    pub size_is_next: bool,
    /// endianness of the struct
    pub endianness: Ident, // TODO enum endianness
    /// magic type and value for this item
    pub magic: Option<(Ident, Lit)>,
    /// skip next item at rest
    pub skip: bool,
    /// context type
    pub context_type: Type,
    /// this field must be used as a context in subsequent read/write operations
    pub is_context: bool,
}

impl Default for Attributes {
    fn default() -> Self {
        Attributes {
            tag_type: None,
            tag: None,
            keep_tag: false,
            keep_diff: None,
            size_type: None,
            byte_sized: false,
            size_is_next: false,
            endianness: Ident::new("NativeEndian", Span::call_site()),
            magic: None,
            skip: false,
            context_type: Type::Verbatim(quote! { () }),
            is_context: false,
        }
    }
}

/// A single Attribute structure makes it easier to write parsing code but give worse error reporting
impl Attributes {
    /// Get structure or enum attributes dedicated to this derive
    pub fn parse(attrs: &Vec<Attribute>) -> Result<Self> {
        let mut result = Attributes::default();
        result._parse(attrs)?;
        Ok(result)
    }

    // sub method of parse and extend
    fn _parse(&mut self, attrs: &Vec<Attribute>) -> Result<()> {
        for attribute in attrs.iter() {
            if !attribute.path().is_ident("plod") {
                continue;
            }
            let meta_parser = syn::meta::parser(|meta| {
                if meta.path.is_ident("tag") {
                    let value = Pat::parse_multi(meta.value()?)?;
                    self.tag = Some(value);
                } else if meta.path.is_ident("keep_diff") {
                    let lit = LitInt::parse(meta.value()?)?;
                    self.keep_diff = Some(lit);
                    self.keep_tag = true;
                } else if meta.path.is_ident("context") {
                    self.context_type = Type::parse(meta.value()?)?;
                } else if meta.path.is_ident("big_endian") {
                    self.endianness = Ident::new("BigEndian", Span::call_site());
                } else if meta.path.is_ident("little_endian") {
                    self.endianness = Ident::new("LittleEndian", Span::call_site());
                } else if meta.path.is_ident("native_endian") {
                    self.endianness = Ident::new("NativeEndian", Span::call_site());
                } else if meta.path.is_ident("keep_tag") {
                    self.keep_tag = true;
                } else if meta.path.is_ident("byte_sized") {
                    self.byte_sized = true;
                } else if meta.path.is_ident("size_is_next") {
                    self.size_is_next = true;
                } else if meta.path.is_ident("skip") {
                    self.skip = true;
                } else if meta.path.is_ident("is_context") {
                    self.is_context = true;
                } else if meta.path.is_ident("magic") {
                    meta.parse_nested_meta(|meta| {
                        let ident = meta.path.get_ident().ok_or(
                            meta.error("Magic must be of the form #[plod(magic(<type>=<value>))]"),
                        )?;
                        let lit = Lit::parse(meta.value()?)?;
                        self.magic = Some((ident.clone(), lit));
                        Ok(())
                    })?;
                } else if meta.path.is_ident("tag_type") {
                    meta.parse_nested_meta(|meta| {
                        self.tag_type = meta.path.get_ident().cloned();
                        Ok(())
                    })?;
                } else if meta.path.is_ident("size_type") {
                    meta.parse_nested_meta(|meta| {
                        self.size_type = meta.path.get_ident().cloned();
                        Ok(())
                    })?;
                } else {
                    return Err(meta.error("Unsupported plod value"));
                }
                Ok(())
            });
            attribute.parse_args_with(meta_parser)?;
        }
        Ok(())
    }

    /// parse attributes that override existing attributes
    pub fn extend(&self, attrs: &Vec<Attribute>) -> Result<Self> {
        let mut result = self.clone();
        // reset non inherited attributes
        result.magic = None;
        result.is_context = false;
        result._parse(attrs)?;
        Ok(result)
    }
}

