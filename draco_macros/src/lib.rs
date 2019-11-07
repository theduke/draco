extern crate proc_macro;

use proc_macro::TokenStream as StdTokenStream;

use proc_macro_hack::proc_macro_hack;
use proc_macro2::TokenStream;
use quote::quote;
use snax::{SnaxAttribute, SnaxItem};

fn render_tag(name: syn::Ident, attrs: Vec<SnaxAttribute>, children: Vec<SnaxItem>, depth: usize) -> TokenStream {
    let attrs = attrs.into_iter().map(|attr| match attr {
        snax::SnaxAttribute::Simple { name: ident, value } => {
            let name = ident.to_string();

            match name.as_str() {
                "disabled" | "checked" => {
                    quote! {
                        .attribute_cond( #name, #value )
                    }
                }
                "oninput" => {
                    quote!{
                        .on_input( #value )
                    }
                }
                "onchecked" => {
                    quote!{
                        .on_checked( #value )
                    }
                }
                event if event.starts_with("on") && event.len() > 2 => {
                    let name = &event[2..];
                    quote!{
                        .on( #name, #value )
                    }
                }
                name => {
                    quote! {
                        .attribute( #name, #value )
                    }
                }
            }
        }
    });

    let children = children.into_iter().map(|child| {

        match child {
            SnaxItem::Fragment(_) => {
                let child_tokens = render_item(child, depth + 1);
                quote! {
                     #child_tokens 
                }
            }
            _ => {
                let child_tokens = render_item(child, depth + 1);
                quote! {
                    .with( #child_tokens )
                }
            }
        }


    });

    let name = name.to_string();
    quote! {
        {
            draco::VNonKeyedElement::new(
                draco::Namespace::Html,
                #name
            )
                #( #attrs )*
                #( #children )*
        }
    }
}

fn render_item(item: snax::SnaxItem, depth: usize) -> TokenStream {
    use snax::SnaxItem::*;
    match item {
        Tag(tag) => render_tag(tag.name, tag.attributes, tag.children, depth),
        SelfClosingTag(tag) => render_tag(tag.name, tag.attributes, Vec::new(), depth),
        Fragment(frag) => {
            if depth == 0 {
                let items = frag.children.into_iter().map(|c| render_item(c, depth + 1));

                quote!{
                    vec![
                        #( #items ),*
                    ]
                }

            } else if frag.children.len() == 1 {
                match frag.children.first().unwrap() {
                    SnaxItem::Content(c) => {
                        quote!{
                            .append( #c )
                        }
                    }
                    _ => {
                        panic!("Nested fragments must contain a single '{}' block");
                    }
                }
            } else {
                panic!("Nested fragments must contain a single '{}' block");
            }
        },
        Content(content) => content.into(),
    }
}

#[proc_macro_hack]
pub fn rsx(tokens: StdTokenStream) -> StdTokenStream {
    let item = snax::parse(tokens.into()).unwrap();
    render_item(item, 0).into()
}
