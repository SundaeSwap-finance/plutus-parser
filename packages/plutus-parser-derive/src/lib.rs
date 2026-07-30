use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{
    Attribute, Data, DeriveInput, Error, Fields, Ident, LitInt, Meta, parse_macro_input,
    spanned::Spanned,
};

#[proc_macro_derive(AsPlutus, attributes(plutus))]
pub fn derive_as_plutus(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let implementation = match &input.data {
        Data::Struct(s) => {
            let format = match get_format(&input.attrs) {
                Ok(Some(f)) => f,
                Ok(None) => DataFormat::Constr { variant: 0 },
                Err(e) => {
                    return e.to_compile_error().into();
                }
            };
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
                            #n: plutus_parser::AsPlutus::from_plutus(#n).map_err(|e| e.with_field_name(stringify!(#n)))?,
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

                    match format {
                        DataFormat::Constr { variant } => {
                            from_plutus = quote! {
                                let (variant, fields) = plutus_parser::parse_constr(data)?;
                                if variant == #variant {
                                    let [#(#names),*] = plutus_parser::parse_variant(variant, fields)?;
                                    return Ok(Self {
                                        #(#assignments)*
                                    });
                                }
                                Err(plutus_parser::DecodeError::unexpected_variant(variant))
                            };
                            to_plutus = quote! {
                                plutus_parser::create_constr(#variant, vec![
                                    #(#casts)*
                                ])
                            };
                        }
                        DataFormat::List => {
                            from_plutus = quote! {
                                let [#(#names),*] = plutus_parser::parse_tuple(data)?;
                                return Ok(Self {
                                    #(#assignments)*
                                });
                            };
                            to_plutus = quote! {
                                plutus_parser::create_array(vec![
                                    #(#casts)*
                                ])
                            };
                        }
                    }
                }
                Fields::Unit => match format {
                    DataFormat::Constr { variant } => {
                        from_plutus = quote! {
                            let (variant, fields) = plutus_parser::parse_constr(data)?;
                            if variant == #variant {
                                let [] = plutus_parser::parse_variant(variant, fields)?;
                                return Ok(Self);
                            }
                            Err(plutus_parser::DecodeError::unexpected_variant(variant))
                        };
                        to_plutus = quote! {
                            plutus_parser::create_constr(#variant, vec![])
                        }
                    }
                    DataFormat::List => {
                        from_plutus = quote! {
                            let [] = plutus_parser::parse_tuple(data)?;
                            return Ok(Self);
                        };
                        to_plutus = quote! {
                            plutus_parser::create_array(vec![])
                        }
                    }
                },
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
                        .enumerate()
                        .map(|(i, n)| {
                            quote! {
                                plutus_parser::AsPlutus::from_plutus(#n).map_err(|e| e.with_field_name(#i))?,
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
                    match format {
                        DataFormat::Constr { variant } => {
                            from_plutus = quote! {
                                let (variant, fields) = plutus_parser::parse_constr(data)?;
                                if variant == #variant {
                                    let [#(#names),*] = plutus_parser::parse_variant(variant, fields)?;
                                    return Ok(Self(#(#assignments)*));
                                }
                                Err(plutus_parser::DecodeError::unexpected_variant(variant))
                            };
                            to_plutus = quote! {
                                let Self(#(#names),*) = self;
                                plutus_parser::create_constr(#variant, vec![
                                    #(#casts)*
                                ])
                            }
                        }
                        DataFormat::List => {
                            from_plutus = quote! {
                                let [#(#names),*] = plutus_parser::parse_tuple(data)?;
                                return Ok(Self(#(#assignments)*));
                            };
                            to_plutus = quote! {
                                let Self(#(#names),*) = self;
                                plutus_parser::create_array(vec![
                                    #(#casts)*
                                ])
                            }
                        }
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
                let n = match get_variant(&variant.attrs) {
                    Ok(Some(variant)) => variant,
                    Ok(None) => seen_variants.len() as u64,
                    Err(e) => {
                        return e.to_compile_error().into();
                    }
                };
                seen_variants.insert(n);
                let (from_clause, to_clause) = match &variant.fields {
                    Fields::Named(named) => {
                        let names: Vec<_> = named
                            .named
                            .iter()
                            .map(|n| n.ident.as_ref().unwrap())
                            .collect();
                        let assignments = names.iter().map(|n| {
                            let field_name = format!("::{name}.{n}");
                            quote! {
                                #n: plutus_parser::AsPlutus::from_plutus(#n).map_err(|e| e.with_field_name(#field_name))?,
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
                            .enumerate()
                            .map(|(i, n)| {
                                let field_name = format!("::{name}.{i}");
                                quote! {
                                    plutus_parser::AsPlutus::from_plutus(#n).map_err(|e| e.with_field_name(#field_name))?,
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
                                return Ok(Self::#name(#(#assignments)*));
                            },
                            quote! {
                                Self::#name(#(#names),*) => plutus_parser::create_constr(#n, vec![
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
                Err(plutus_parser::DecodeError::unexpected_variant(variant))
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

enum DataFormat {
    List,
    Constr { variant: u64 },
}

fn get_format(attrs: &[Attribute]) -> Result<Option<DataFormat>, Error> {
    for a in attrs {
        let Meta::List(list) = &a.meta else {
            continue;
        };
        if !list.path.is_ident("plutus") {
            continue;
        }
        let mut format = None;
        list.parse_nested_meta(|meta| {
            if meta.path.is_ident("list") {
                format = Some(DataFormat::List);
                Ok(())
            } else if meta.path.is_ident("constr") {
                if !meta.input.is_empty() {
                    let value = meta.value()?;
                    let i: LitInt = value.parse()?;
                    format = Some(DataFormat::Constr {
                        variant: i.base10_parse()?,
                    });
                }
                Ok(())
            } else {
                Err(Error::new(meta.input.span(), "unrecognized field"))
            }
        })?;
        return Ok(format);
    }
    Ok(None)
}

fn get_variant(attrs: &[Attribute]) -> Result<Option<u64>, Error> {
    for a in attrs {
        let Meta::List(list) = &a.meta else {
            continue;
        };
        if !list.path.is_ident("plutus") {
            continue;
        }
        let mut variant = None;
        list.parse_nested_meta(|meta| {
            if meta.path.is_ident("constr") {
                let value = meta.value()?;
                let i: LitInt = value.parse()?;
                variant = Some(i.base10_parse()?);
                Ok(())
            } else {
                Err(Error::new(meta.input.span(), "unrecognized field"))
            }
        })?;
        return Ok(variant);
    }
    Ok(None)
}
