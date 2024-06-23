use proc_macro::TokenStream;
use quote::{ quote, ToTokens };
use syn::{ spanned::Spanned, DeriveInput, Ident };

#[proc_macro_derive(RegisterField)]
pub fn derive_register_field(item: TokenStream) -> TokenStream {
    match derive_register_field_internal(item) {
        Ok(tokens) => tokens,
        Err(e) => e.into_compile_error().into(),
    }
}

fn derive_register_field_internal(input: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse::<DeriveInput>(input)?;

    let enum_data = (match input.data {
        syn::Data::Enum(data) => Ok(data),
        syn::Data::Struct(_) => Err(syn::Error::new(input.span(), "Struct is not supported")),
        syn::Data::Union(_) => Err(syn::Error::new(input.span(), "Union is not supported")),
    })?;

    let name = input.ident;
    let mut entries = Vec::new();

    let mut discriminant = 0u32;
    for variant in enum_data.variants {
        if let Some(d) = variant.discriminant {
            let d = (match d.1 {
                syn::Expr::Lit(lit) =>
                    match lit.lit {
                        syn::Lit::Int(i) => Ok(i.base10_parse::<u32>()?),
                        e => Err(syn::Error::new(e.span(), "not an integer literal")),
                    }
                e => Err(syn::Error::new(e.span(), "not an integer literal")),
            })?;

            discriminant = d;
        }

        entries.push(MatchEntry::new(variant.ident, discriminant));
        discriminant += 1;
    }

    Ok(
        (
            quote! {
                impl ::register::field::RegisterField for #name {
                    #[inline(always)]
                    fn into_bits(self) -> u32 {
                        self as _
                    }

                    #[inline(always)]
                    fn from_bits(value: u32) -> Self {
                        match value {
                            #( #entries )*
                            _ => panic!()
                        }
                    }
                }
            }
        ).into()
    )
}

struct MatchEntry {
    variant: Ident,
    discriminant: u32,
}

impl MatchEntry {
    fn new(variant: Ident, discriminant: u32) -> Self {
        Self { variant, discriminant }
    }
}

impl ToTokens for MatchEntry {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let disc = self.discriminant;
        let variant = self.variant.clone();

        tokens.extend(quote! {
            #disc => Self::#variant,
        })
    }
}
