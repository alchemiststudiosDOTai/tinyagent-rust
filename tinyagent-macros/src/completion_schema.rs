use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, ItemStruct, LitStr};

use crate::schema_extraction::{
    collect_doc_comments, collect_field_docs, ensure_named_struct, infer_description,
    infer_schema_name, parse_completion_schema_args,
};

pub fn completion_schema(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = match parse_completion_schema_args(attr) {
        Ok(args) => args,
        Err(err) => return err.to_compile_error().into(),
    };

    let item_struct = parse_macro_input!(item as ItemStruct);

    if let Err(err) = ensure_named_struct(&item_struct) {
        return err.to_compile_error().into();
    }

    if !item_struct.generics.params.is_empty() {
        return syn::Error::new(
            item_struct.generics.span(),
            "`#[completion_schema]` does not currently support generic structs",
        )
        .to_compile_error()
        .into();
    }

    let schema_name = infer_schema_name(&item_struct, args.name.as_ref());
    let struct_docs = collect_doc_comments(&item_struct.attrs);
    let description = infer_description(args.description.as_ref(), struct_docs);

    let description_tokens = description
        .as_ref()
        .map(|lit| quote! { Some(#lit) })
        .unwrap_or_else(|| quote! { None });

    let field_docs = collect_field_docs(&item_struct);
    let field_doc_tokens: Vec<_> = field_docs
        .iter()
        .map(|(field, doc)| {
            let field_lit = LitStr::new(field, Span::call_site());
            let doc_lit = LitStr::new(doc, Span::call_site());
            quote! { (#field_lit, #doc_lit) }
        })
        .collect();

    let type_name = LitStr::new(&item_struct.ident.to_string(), Span::call_site());
    let ident = &item_struct.ident;

    let expanded = quote! {
        #item_struct

        impl tiny_agent_rs::schema::CompletionSchema for #ident {
            fn schema() -> &'static tiny_agent_rs::schema::SchemaHandle {
                static HANDLE: std::sync::OnceLock<tiny_agent_rs::schema::SchemaHandle> = std::sync::OnceLock::new();
                HANDLE.get_or_init(|| {
                    let mut root = schemars::schema_for!(Self);
                    tiny_agent_rs::schema::apply_doc_comments(
                        &mut root,
                        #schema_name,
                        #description_tokens,
                        &[#(#field_doc_tokens),*],
                    );
                    tiny_agent_rs::schema::SchemaHandle::from_root_schema::<Self>(
                        #schema_name,
                        #type_name,
                        root,
                    )
                })
            }
        }
    };

    expanded.into()
}
