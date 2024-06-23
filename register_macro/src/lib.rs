use proc_macro2::TokenStream;
use quote::{ quote, format_ident, ToTokens };
use syn::{ spanned::Spanned, parse::{ Parse, ParseStream } };

#[proc_macro_attribute]
pub fn register(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    match register_inner(args.into(), input.into()) {
        Ok(result) => result.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn register_inner(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let input: syn::ItemStruct = syn::parse2(input)?;
    let attrs: RegisterAttrs = syn::parse2(args)?;

    let base_typ = attrs.typ;
    let name = input.ident.clone();
    let vis = input.vis.clone();
    let attrs = input.attrs.iter().map(ToTokens::to_token_stream).collect::<TokenStream>();

    let syn::Type::Path(base_path) = base_typ.clone() else {
        return Err(syn::Error::new(input.span(), "invalid base type"));
    };

    let syn::Fields::Named(fields) = input.fields.clone() else {
        return Err(syn::Error::new(input.fields.span(), "only named fields are supported"));
    };

    let base_bits = if base_path.path.is_ident("u8") {
        u8::BITS
    } else if base_path.path.is_ident("u16") {
        u16::BITS
    } else if base_path.path.is_ident("u32") {
        u32::BITS
    } else {
        return Err(syn::Error::new(input.fields.span(), "unsupported base type"));
    };

    let mut offset = 0;
    let mut bits = Vec::new();
    for f in fields.named {
        let reg = RegBits::new(offset, &f, &base_typ, base_bits)?;
        offset += reg.attrs.bits;

        if f.ident.clone().unwrap() != "__" {
            bits.push(reg);
        }
    }
    if offset > u32::BITS {
        return Err(syn::Error::new(input.span(), "overflowing register type"));
    }

    Ok(
        quote! {
            #attrs
            #vis struct #name(core::cell::UnsafeCell<#base_typ>);

            impl #name {
                #vis fn get(&self) -> #base_typ {
                    unsafe { core::ptr::read_volatile(self.0.get()) }
                }

                #vis fn set(&mut self, val: #base_typ) {
                    unsafe { core::ptr::write_volatile(self.0.get(), val) }
                }

                #( #bits )*
            }
        }
    )
}

struct RegBits {
    offset: u32,
    base_typ: syn::Type,
    base_bits: u32,
    vis: syn::Visibility,
    typ: syn::Type,
    ident: syn::Ident,
    attrs: RegBitsAttrs,
    from: TokenStream,
    into: TokenStream,
}

impl RegBits {
    fn new(offset: u32, f: &syn::Field, base_typ: &syn::Type, base_bits: u32) -> syn::Result<Self> {
        let typ = f.ty.clone();
        let ident = f.ident.clone().unwrap();

        let mut ret = Self {
            offset,
            base_typ: base_typ.clone(),
            base_bits,
            vis: f.vis.clone(),
            typ: typ.clone(),
            ident,
            attrs: RegBitsAttrs::new(),
            from: quote!(),
            into: quote!(),
        };

        for attr in &f.attrs {
            let syn::Attribute {
                style: syn::AttrStyle::Outer,
                meta: syn::Meta::List(syn::MetaList { path, tokens, .. }),
                ..
            } = attr else {
                continue;
            };

            if path.is_ident("bits") {
                ret.attrs = syn::parse2::<RegBitsAttrs>(tokens.clone())?;
            }
        }

        let syn::Type::Path(path) = typ.clone() else {
            return Err(syn::Error::new(f.span(), "invalid bits type"));
        };

        if ret.attrs.bits == 0 {
            return Err(syn::Error::new(f.span(), "bits cannot be 0"));
        }
        if
            (path.path.is_ident("bool") && ret.attrs.bits > 1) ||
            (path.path.is_ident("u8") && ret.attrs.bits > u8::BITS) ||
            (path.path.is_ident("u16") && ret.attrs.bits > u16::BITS) ||
            (path.path.is_ident("u32") && ret.attrs.bits > u32::BITS)
        {
            return Err(syn::Error::new(f.span(), "overflowing field type"));
        }

        ret.from = if let Some(from) = ret.attrs.from.as_ref() {
            quote!(#from(val))
        } else {
            if path.path.is_ident("bool") {
                quote!(val != 0)
            } else if
                path.path.is_ident("u8") ||
                path.path.is_ident("u16") ||
                path.path.is_ident("u32")
            {
                quote!(val as _)
            } else {
                quote!(::register::field::RegisterField::from_bits(val))
            }
        };

        ret.into = if let Some(into) = ret.attrs.into.as_ref() {
            quote!(#into(val))
        } else {
            if path.path.is_ident("bool") {
                quote!(if val {1} else {0})
            } else if
                path.path.is_ident("u8") ||
                path.path.is_ident("u16") ||
                path.path.is_ident("u32")
            {
                quote!(val as _)
            } else {
                quote!(::register::field::RegisterField::into_bits(val))
            }
        };

        Ok(ret)
    }
}

impl ToTokens for RegBits {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.ident.clone();
        let get_ident = self.attrs.get.clone().unwrap_or(format_ident!("get_{ident}"));
        let set_ident = self.attrs.set.clone().unwrap_or(format_ident!("set_{ident}"));
        let clear_ident = self.attrs.clear.clone().unwrap_or(format_ident!("clear_{ident}"));

        let base_typ = &self.base_typ;
        let vis = &self.vis;
        let typ = &self.typ;
        let from = self.from.clone();
        let into = self.into.clone();

        let offset = self.offset;

        let mask: u32 = (u32::MAX >> (self.base_bits - self.attrs.bits)) << offset;

        let is_bool = match typ {
            syn::Type::Path(p) => p.path.is_ident("bool"),
            _ => false,
        };

        let access = self.attrs.access.to_string();

        if access.contains('r') {
            tokens.extend(
                quote! {
                    #[inline(always)]
                    #vis fn #get_ident(&self) -> #typ {
                        let val: #base_typ = ((unsafe { core::ptr::read_volatile(self.0.get()) }) & #mask) >> #offset;
                        #from
                    }
                }
            );
        }

        if access.contains('w') {
            if is_bool && access.contains('c') {
                tokens.extend(
                    quote! {
                        #[inline(always)]
                        #vis fn #set_ident(&mut self) {
                            unsafe {
                                let reg = core::ptr::read_volatile(self.0.get());
                                core::ptr::write_volatile(self.0.get(), reg | #mask);
                            }
                        }
                    }
                );
            } else {
                tokens.extend(
                    quote! {
                        #[inline(always)]
                        #vis fn #set_ident(&mut self, val: #typ) {
                            let val: #base_typ = #into;
                            unsafe {
                                let reg = core::ptr::read_volatile(self.0.get());
                                core::ptr::write_volatile(self.0.get(), (reg & !#mask) | (val << #offset));
                            }
                        }
                    }
                );
            }
        }

        if access.contains('c') {
            tokens.extend(
                quote! {
                    #[inline(always)]
                    #vis fn #clear_ident(&mut self) {
                        unsafe {
                            let reg = core::ptr::read_volatile(self.0.get());
                            core::ptr::write_volatile(self.0.get(), reg & !#mask);
                        }
                    }
                }
            )
        }
    }
}

struct RegisterAttrs {
    typ: syn::Type,
    // bits: usize,
}

impl Parse for RegisterAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let Ok(typ) = syn::Type::parse(input) else {
            return Err(syn::Error::new(input.span(), "unknown type"));
        };

        Ok(Self {
            typ,
        })
    }
}

struct RegBitsAttrs {
    bits: u32,
    access: syn::Ident,
    get: Option<syn::Ident>,
    set: Option<syn::Ident>,
    clear: Option<syn::Ident>,
    from: Option<syn::Path>,
    into: Option<syn::Path>,
}

impl RegBitsAttrs {
    fn new() -> Self {
        Self {
            bits: 1,
            access: format_ident!("rw"),
            get: None,
            set: None,
            clear: None,
            from: None,
            into: None,
        }
    }
}

impl Parse for RegBitsAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attrs = Self::new();
        if let Ok(bits) = syn::LitInt::parse(input) {
            attrs.bits = bits.base10_parse()?;
        }

        while !input.is_empty() {
            syn::token::Comma::parse(input)?;

            let ident = syn::Ident::parse(input)?;

            if
                ident == "r" ||
                ident == "rw" ||
                ident == "rwc" ||
                ident == "rc" ||
                ident == "w" ||
                ident == "wc"
            {
                attrs.access = ident;
            } else if ident == "get" {
                syn::token::Eq::parse(input)?;
                attrs.get = Some(syn::Ident::parse(input)?);
            } else if ident == "set" {
                syn::token::Eq::parse(input)?;
                attrs.set = Some(syn::Ident::parse(input)?);
            } else if ident == "clear" {
                syn::token::Eq::parse(input)?;
                attrs.clear = Some(syn::Ident::parse(input)?);
            } else if ident == "from" {
                syn::token::Eq::parse(input)?;
                attrs.from = Some(syn::Path::parse(input)?);
            } else if ident == "into" {
                syn::token::Eq::parse(input)?;
                attrs.into = Some(syn::Path::parse(input)?);
            } else {
                return Err(syn::Error::new(ident.span(), "unsupported access type"));
            }
        }

        Ok(attrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
