/*
    dumpster, a cycle-tracking garbage collector for Rust.
    Copyright (C) 2023 Clayton Ramsey.

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Path, Result};

synstructure::decl_derive!(
    [Trace, attributes(dumpster)] =>
    /// Derive `Trace` for a type.
    derive_trace
);

fn derive_trace(mut s: synstructure::Structure) -> Result<TokenStream> {
    let mut dumpster: Path = parse_quote!(::dumpster);
    let mut trace_ignore_container = false;

    // look for container attributes
    for attr in &s.ast().attrs {
        if !attr.path().is_ident("dumpster") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("crate") {
                dumpster = meta.value()?.parse()?;
                Ok(())
            } else if meta.path.is_ident("trace") {
                meta.parse_nested_meta(|meta| {
                    if meta.path.is_ident("ignore") {
                        trace_ignore_container = true;
                        Ok(())
                    } else {
                        Err(meta.error("unsupported trace attribute"))
                    }
                })
            } else {
                Err(meta.error("unsupported attribute"))
            }
        })?;
    }

    let body = if trace_ignore_container {
        // With `trace(ignore)` no additional bounds are added.
        s.add_bounds(synstructure::AddBounds::None);
        quote!()
    } else {
        // Every field must implement `Trace` (but not necessarily the generics).
        s.add_bounds(synstructure::AddBounds::Fields);

        // There is no `try_filter` so we store the parse error here, to return it
        // after the `filter` call.
        let mut field_attr_parse_error = None;

        // Filter out fields with `#[dumpster(trace(ignore))]`
        s.filter(|bi| {
            let mut trace_ignore = false;

            for attr in &bi.ast().attrs {
                if !attr.path().is_ident("dumpster") {
                    continue;
                }

                let result = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("trace") {
                        meta.parse_nested_meta(|meta| {
                            if meta.path.is_ident("ignore") {
                                trace_ignore = true;
                                Ok(())
                            } else {
                                Err(meta.error("unsupported trace attribute argument"))
                            }
                        })
                    } else {
                        Err(meta.error("unsupported attribute"))
                    }
                });

                if let Err(error) = result {
                    field_attr_parse_error.get_or_insert(error);
                }
            }

            !trace_ignore
        });

        if let Some(error) = field_attr_parse_error {
            return Err(error);
        }

        let body = s.each(|bi| {
            quote! {
                #dumpster::TraceWith::accept(#bi, visitor)?;
            }
        });

        quote!(match *self { #body })
    };

    Ok(s.gen_impl(quote! {
        gen unsafe impl<__V: #dumpster::Visitor> #dumpster::TraceWith<__V> for @Self {
            #[inline]
            fn accept(&self, visitor: &mut __V) -> ::core::result::Result<(), ()> {
                #body
                ::core::result::Result::Ok(())
            }
        }
    }))
}
