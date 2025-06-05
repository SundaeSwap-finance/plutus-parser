use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, ExprLit, Fields, Ident, Lit, Meta,
    parse_macro_input, spanned::Spanned,
};

#[proc_macro_derive(AsPlutus)]
pub fn derive_as_plutus(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let implementation = match &input.data {
        Data::Struct(s) => {
            let n = get_variant(&input.attrs).unwrap_or_default();
            let from_plutus;
            let to_plutus;
            match &s.fields {
                Fields::Named(named) => {
                    let names: Vec<_> = named
                        .named
                        .iter()
                        .map(|n| n.ident.as_ref().unwrap())
                        .collect();
                    let assignments = names.iter().map(|n| {
                        quote! {
                            #n: plutus_parser::AsPlutus::from_plutus(#n)?,
                        }
                    });
                    let casts: Vec<_> = names
                        .iter()
                        .map(|n| {
                            quote! {
                                self.#n.to_plutus(),
                            }
                        })
                        .collect();

                    from_plutus = quote! {
                        let (variant, fields) = plutus_parser::parse_constr(data)?;
                        if variant == #n {
                            let [#(#names),*] = plutus_parser::parse_variant(variant, fields)?;
                            return Ok(Self {
                                #(#assignments)*
                            });
                        }
                        Err(plutus_parser::DecodeError::UnexpectedVariant { variant })
                    };
                    to_plutus = quote! {
                        plutus_parser::create_constr(#n, vec![
                            #(#casts)*
                        ])
                    };
                }
                Fields::Unit => {
                    from_plutus = quote! {
                        let (variant, fields) = plutus_parser::parse_constr(data)?;
                        if variant == #n {
                            let [] = plutus_parser::parse_variant(variant, fields)?;
                            return Ok(Self);
                        }
                        Err(plutus_parser::DecodeError::UnexpectedVariant { variant })
                    };
                    to_plutus = quote! {
                        plutus_parser::create_constr(#n, vec![])
                    }
                }
                Fields::Unnamed(fields) => {
                    let names: Vec<_> = fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(i, field)| {
                            let name = format!("f{i}");
                            let span = field.span();
                            Ident::new(&name, span)
                        })
                        .collect();
                    let assignments: Vec<_> = names
                        .iter()
                        .map(|n| {
                            quote! {
                                plutus_parser::AsPlutus::from_plutus(#n)?,
                            }
                        })
                        .collect();
                    let casts: Vec<_> = names
                        .iter()
                        .map(|n| {
                            quote! {
                                #n.to_plutus(),
                            }
                        })
                        .collect();
                    from_plutus = quote! {
                        let (variant, fields) = plutus_parser::parse_constr(data)?;
                        if variant == #n {
                            let [#(#names),*] = plutus_parser::parse_variant(variant, fields)?;
                            return Ok(Self(#(#assignments)*));
                        }
                        Err(plutus_parser::DecodeError::UnexpectedVariant { variant })
                    };
                    to_plutus = quote! {
                        let Self(#(#names),*) = self;
                        plutus_parser::create_constr(#n, vec![
                            #(#casts)*
                        ])
                    }
                }
            };

            quote! {
                fn from_plutus(data: plutus_parser::PlutusData) -> Result<Self, plutus_parser::DecodeError> {
                    #from_plutus
                }

                fn to_plutus(self) -> plutus_parser::PlutusData {
                    #to_plutus
                }
            }
        }
        Data::Enum(e) => {
            let mut from_plutus = quote! {
                let (variant, fields) = plutus_parser::parse_constr(data)?;
            };
            let mut to_plutus = quote! {};
            let mut seen_variants = HashSet::new();
            for variant in &e.variants {
                let name = &variant.ident;
                let n = get_variant(&variant.attrs).unwrap_or(seen_variants.len() as u64);
                seen_variants.insert(n);
                let (from_clause, to_clause) = match &variant.fields {
                    Fields::Named(named) => {
                        let names: Vec<_> = named
                            .named
                            .iter()
                            .map(|n| n.ident.as_ref().unwrap())
                            .collect();
                        let assignments = names.iter().map(|n| {
                            quote! {
                                #n: plutus_parser::AsPlutus::from_plutus(#n)?,
                            }
                        });
                        let casts: Vec<_> = names
                            .iter()
                            .map(|n| {
                                quote! {
                                    #n.to_plutus(),
                                }
                            })
                            .collect();
                        (
                            quote! {
                                let [#(#names),*] = plutus_parser::parse_variant(variant, fields)?;
                                return Ok(Self::#name {
                                    #(#assignments)*
                                });
                            },
                            quote! {
                                Self::#name { #(#names),* } => plutus_parser::create_constr(#n, vec![
                                    #(#casts)*
                                ]),
                            },
                        )
                    }
                    Fields::Unit => (
                        quote! {
                            let [] = plutus_parser::parse_variant(variant, fields)?;
                            return Ok(Self::#name);
                        },
                        quote! {
                            Self::#name => plutus_parser::create_constr(#n, vec![]),
                        },
                    ),
                    Fields::Unnamed(fields) => {
                        let names: Vec<_> = fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, field)| {
                                let name = format!("f{i}");
                                let span = field.span();
                                Ident::new(&name, span)
                            })
                            .collect();
                        let assignments: Vec<_> = names
                            .iter()
                            .map(|n| {
                                quote! {
                                    plutus_parser::AsPlutus::from_plutus(#n)?,
                                }
                            })
                            .collect();
                        let casts: Vec<_> = names
                            .iter()
                            .map(|n| {
                                quote! {
                                    #n.to_plutus(),
                                }
                            })
                            .collect();
                        (
                            quote! {
                                let [#(#names),*] = plutus_parser::parse_variant(variant, fields)?;
                                return Ok(Self::#name(#(#assignments),*));
                            },
                            quote! {
                                Self::#name(#(#names)*) => plutus_parser::create_constr(#n, vec![
                                    #(#casts)*
                                ]),
                            },
                        )
                    }
                };
                from_plutus.extend(quote_spanned! { variant.span() =>
                    if variant == #n {
                        #from_clause
                    }
                });
                to_plutus.extend(quote_spanned! {variant.span() =>
                    #to_clause
                });
            }
            from_plutus.extend(quote! {
                Err(plutus_parser::DecodeError::UnexpectedVariant { variant })
            });

            quote! {
                fn from_plutus(data: plutus_parser::PlutusData) -> Result<Self, plutus_parser::DecodeError> {
                    #from_plutus
                }

                fn to_plutus(self) -> plutus_parser::PlutusData {
                    match self {
                        #to_plutus
                    }
                }
            }
        }
        _ => {
            return Error::new(Span::call_site(), "Unsupported type")
                .into_compile_error()
                .into();
        }
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics plutus_parser::AsPlutus for #name #ty_generics #where_clause {
            #implementation
        }
    };

    TokenStream::from(expanded)
}

fn get_variant(attrs: &[Attribute]) -> Option<u64> {
    attrs.iter().find_map(|a| {
        let Meta::NameValue(name_value) = &a.meta else {
            return None;
        };
        if !name_value.path.is_ident("variant") {
            return None;
        }
        let Expr::Lit(ExprLit {
            lit: Lit::Int(int), ..
        }) = &name_value.value
        else {
            return None;
        };
        int.base10_parse().ok()
    })
}
