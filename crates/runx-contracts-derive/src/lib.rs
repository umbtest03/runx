//! `#[derive(RunxSchema)]`: emit a wire-compatible JSON Schema document from a
//! contract type's definition and its serde attributes. Part of Phase 1 of
//! `rust-contract-pipeline-inversion`.
//!
//! Supported today: named-field structs (honoring `#[serde(rename)]`,
//! `#[serde(rename_all)]`, `#[serde(skip)]`, `Option<T>` optionality, and
//! `#[serde(deny_unknown_fields)]`) and unit-only enums (rendered as `anyOf` of
//! `const`). Data-carrying enums emit a clear compile error until their oneOf
//! shape lands in a later batch.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Fields, GenericArgument, Lit, PathArguments, Type, parse_macro_input,
};

#[proc_macro_derive(RunxSchema, attributes(runx_schema))]
pub fn derive_runx_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    let identity = runx_identity(&input.attrs)?;
    let identity_expr = match identity {
        Some(logical) => quote! { ::core::option::Option::Some(#logical) },
        None => quote! { ::core::option::Option::None },
    };

    let body = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(_) => struct_body(input, data, &identity_expr)?,
            _ => {
                return Err(syn::Error::new_spanned(
                    ident,
                    "RunxSchema supports only named-field structs",
                ));
            }
        },
        Data::Enum(data) => enum_body(input, data)?,
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                ident,
                "RunxSchema cannot be derived for unions",
            ));
        }
    };

    Ok(quote! {
        impl ::runx_contracts::schema::RunxSchema for #ident {
            fn json_schema() -> ::serde_json::Value {
                #body
            }
        }
    })
}

fn struct_body(
    input: &DeriveInput,
    data: &syn::DataStruct,
    identity_expr: &proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let rename_all = serde_rename_all(&input.attrs)?;
    let deny_unknown = serde_deny_unknown_fields(&input.attrs);

    let mut properties = Vec::new();
    for field in &data.fields {
        let Some(ident) = field.ident.as_ref() else {
            continue;
        };
        if serde_skip(&field.attrs) {
            continue;
        }
        let wire_name = match serde_rename(&field.attrs)? {
            Some(name) => name,
            None => apply_rename_all(&ident.to_string(), rename_all.as_deref()),
        };
        let (inner_ty, optional) = unwrap_option(&field.ty);
        let required = !optional;
        properties.push(quote! {
            ::runx_contracts::schema::Property::new(
                #wire_name,
                <#inner_ty as ::runx_contracts::schema::RunxSchema>::json_schema(),
                #required,
            )
        });
    }

    Ok(quote! {
        ::runx_contracts::schema::object_schema(
            ::std::vec![#(#properties),*],
            #deny_unknown,
            #identity_expr,
        )
    })
}

fn enum_body(input: &DeriveInput, data: &syn::DataEnum) -> syn::Result<proc_macro2::TokenStream> {
    let rename_all = serde_rename_all(&input.attrs)?;
    let mut names = Vec::new();
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                &variant.ident,
                "RunxSchema enum support is unit-only for now (data variants land in a later batch)",
            ));
        }
        let wire_name = match serde_rename(&variant.attrs)? {
            Some(name) => name,
            None => apply_rename_all(&variant.ident.to_string(), rename_all.as_deref()),
        };
        names.push(wire_name);
    }
    Ok(quote! {
        ::runx_contracts::schema::string_enum(&[#(#names),*])
    })
}

/// Strip a leading `Option<...>`, returning the inner type and whether it was
/// optional.
fn unwrap_option(ty: &Type) -> (Type, bool) {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner)) = args.args.first() {
                        return (inner.clone(), true);
                    }
                }
            }
        }
    }
    (ty.clone(), false)
}

/// `#[runx_schema(id = "runx.reference.v1")]` on the type, if present.
fn runx_identity(attrs: &[syn::Attribute]) -> syn::Result<Option<String>> {
    for attr in attrs {
        if !attr.path().is_ident("runx_schema") {
            continue;
        }
        let mut found = None;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("id") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                found = Some(lit.value());
                Ok(())
            } else {
                Err(meta.error("unsupported runx_schema attribute"))
            }
        })?;
        if found.is_some() {
            return Ok(found);
        }
    }
    Ok(None)
}

fn serde_rename_all(attrs: &[syn::Attribute]) -> syn::Result<Option<String>> {
    serde_string_value(attrs, "rename_all")
}

fn serde_rename(attrs: &[syn::Attribute]) -> syn::Result<Option<String>> {
    serde_string_value(attrs, "rename")
}

/// Read a `#[serde(<key> = "value")]` string value if present.
fn serde_string_value(attrs: &[syn::Attribute], key: &str) -> syn::Result<Option<String>> {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let mut found = None;
        let parsed = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(key) {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                found = Some(lit.value());
                Ok(())
            } else {
                // Consume any value for keys we do not care about so parsing
                // does not error on `rename = "x"`, `default`, etc.
                if let Ok(value) = meta.value() {
                    let _: Lit = value.parse()?;
                }
                Ok(())
            }
        });
        // Ignore serde forms we do not model (e.g. bare path flags); only the
        // requested string value matters here.
        if parsed.is_ok() && found.is_some() {
            return Ok(found);
        }
    }
    Ok(None)
}

fn serde_flag(attrs: &[syn::Attribute], flag: &str) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let mut present = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(flag) {
                present = true;
            } else if let Ok(value) = meta.value() {
                let _: Lit = value.parse()?;
            }
            Ok(())
        });
        if present {
            return true;
        }
    }
    false
}

fn serde_deny_unknown_fields(attrs: &[syn::Attribute]) -> bool {
    serde_flag(attrs, "deny_unknown_fields")
}

fn serde_skip(attrs: &[syn::Attribute]) -> bool {
    serde_flag(attrs, "skip") || serde_flag(attrs, "skip_serializing")
}

/// Apply a serde `rename_all` rule to an identifier. Covers the rules the
/// contract types use.
fn apply_rename_all(ident: &str, rule: Option<&str>) -> String {
    match rule {
        Some("snake_case") => to_snake_case(ident),
        Some("camelCase") => to_camel_case(ident),
        Some("PascalCase") => to_pascal_case(ident),
        Some("kebab-case") => to_snake_case(ident).replace('_', "-"),
        Some("SCREAMING_SNAKE_CASE") => to_snake_case(ident).to_uppercase(),
        _ => ident.to_owned(),
    }
}

fn to_snake_case(ident: &str) -> String {
    let mut out = String::new();
    for (index, ch) in ident.chars().enumerate() {
        if ch.is_uppercase() {
            if index != 0 {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn to_pascal_case(ident: &str) -> String {
    let mut out = String::new();
    let mut upper_next = true;
    for ch in ident.chars() {
        if ch == '_' {
            upper_next = true;
        } else if upper_next {
            out.extend(ch.to_uppercase());
            upper_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

fn to_camel_case(ident: &str) -> String {
    let pascal = to_pascal_case(ident);
    let mut chars = pascal.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => pascal,
    }
}
