use proc_macro2::Span;
use syn::{
    parse::Parser, punctuated::Punctuated, spanned::Spanned, Attribute, Expr, ExprLit, Fields,
    ItemStruct, Lit, LitStr, MetaNameValue, Token,
};

#[derive(Default)]
pub struct CompletionSchemaArgs {
    pub name: Option<LitStr>,
    pub description: Option<LitStr>,
}

pub fn parse_completion_schema_args(
    attr: proc_macro::TokenStream,
) -> syn::Result<CompletionSchemaArgs> {
    if attr.is_empty() {
        return Ok(CompletionSchemaArgs::default());
    }

    let parser = Punctuated::<MetaNameValue, Token![,]>::parse_terminated;
    let args = parser.parse(attr)?;

    let mut result = CompletionSchemaArgs::default();

    for nested in args {
        let ident = nested
            .path
            .get_ident()
            .ok_or_else(|| syn::Error::new_spanned(&nested.path, "expected identifier"))?;

        let lit_str = match &nested.value {
            Expr::Lit(ExprLit {
                lit: Lit::Str(lit), ..
            }) => lit.clone(),
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "expected string literal value",
                ));
            }
        };

        match ident.to_string().as_str() {
            "name" => {
                if result.name.is_some() {
                    return Err(syn::Error::new(ident.span(), "duplicate `name` argument"));
                }
                result.name = Some(lit_str);
            }
            "description" => {
                if result.description.is_some() {
                    return Err(syn::Error::new(
                        ident.span(),
                        "duplicate `description` argument",
                    ));
                }
                result.description = Some(lit_str);
            }
            other => {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("unsupported argument `{other}`"),
                ));
            }
        }
    }

    Ok(result)
}

pub fn ensure_named_struct(item: &ItemStruct) -> syn::Result<()> {
    match &item.fields {
        Fields::Named(_) => Ok(()),
        _ => Err(syn::Error::new(
            item.struct_token.span(),
            "`#[completion_schema]` only supports structs with named fields",
        )),
    }
}

pub fn collect_doc_comments(attrs: &[Attribute]) -> Option<String> {
    let mut docs = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Ok(lit) = attr.parse_args::<LitStr>() {
                docs.push(lit.value().trim().to_string());
            }
        }
    }

    if docs.is_empty() {
        None
    } else {
        Some(docs.join("\n"))
    }
}

pub fn collect_field_docs(item: &ItemStruct) -> Vec<(String, String)> {
    let mut results = Vec::new();

    if let Fields::Named(fields) = &item.fields {
        for field in &fields.named {
            if let Some(ident) = &field.ident {
                if let Some(doc) = collect_doc_comments(&field.attrs) {
                    results.push((ident.to_string(), doc));
                }
            }
        }
    }

    results
}

pub fn infer_schema_name(item: &ItemStruct, explicit: Option<&LitStr>) -> LitStr {
    if let Some(explicit) = explicit {
        return explicit.clone();
    }

    LitStr::new(&item.ident.to_string(), Span::call_site())
}

pub fn infer_description(explicit: Option<&LitStr>, doc: Option<String>) -> Option<LitStr> {
    if let Some(explicit) = explicit {
        return Some(explicit.clone());
    }

    doc.map(|text| LitStr::new(&text, Span::call_site()))
}
