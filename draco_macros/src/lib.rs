extern crate proc_macro;

mod parser;
mod builder;

use proc_macro::TokenStream;
use quote::quote;


#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    use std::str::FromStr;

    let raw = input.to_string();
    let doc = match parser::parse_doc(&raw) {
        Ok(doc) => doc,
        Err(e) => {
            panic!("Parsing error: \n{}", e);
        }
    };
    let output = builder::build(doc);
    // println!("OUTPUT\n\n{}\n\n", output);
    let tokens = TokenStream::from_str(&output)
        .expect("Internal error: code-generation produced invalid tokens");
    tokens
}

struct Html {
    pub code: proc_macro2::TokenStream,
}

impl syn::parse::Parse for Html {
    fn parse(input: &syn::parse::ParseBuffer) -> syn::parse::Result<Self> {
        use syn::{Ident, LitStr, Token};

        // Opening bracket.
        input.parse::<Token![<]>()?;

        // Tag name.

        let tag = input.parse::<Ident>()?;

        // attributes.
        let mut attr_code = Vec::new();
        loop {
            if let Ok(attr_name) = input.parse::<Ident>() {
                // EQ.
                input.parse::<Token![=]>()?;
                // Value.
                // Either raw string literal or bracket expression.

                let attr_name_str = attr_name.to_string();

                // Check for event handlers.
                if attr_name_str.starts_with("on") && attr_name_str.len() > 2 {
                    let mut chars = attr_name_str.chars().skip(2);

                    let handler_name = format!(
                        "{}{}",
                        chars.next().unwrap().to_lowercase(),
                        chars.collect::<String>()
                    );

                    let handler_block = input.parse::<syn::Block>()
                        .expect(&format!(
                            "Expected a block expression for attribute {} in tag {}\nExample: <{} {}={{...}}>", 
                            attr_name_str, tag, tag, attr_name_str)
                        );
                    if handler_name == "input" {
                        attr_code.push(quote!(
                            .on_input(#handler_block)
                        ));
                    } else if handler_name == "checked" {
                        attr_code.push(quote!(
                            .on_checked(#handler_block)
                        ));
                    } else {
                        attr_code.push(quote!(
                            .on(#handler_name, #handler_block)
                        ));
                    }
                } else {
                    // Fix up the name.
                    let mut attr_css_name = String::new();
                    for c in attr_name_str.chars() {
                        if c.is_uppercase() {
                            attr_css_name.push('-');
                            for cc in c.to_lowercase() {
                                attr_css_name.push(cc);
                            }
                        } else {
                            attr_css_name.push(c);
                        }
                    }
                    let value = if let Ok(str_lit) = input.parse::<LitStr>() {
                        syn::Expr::Lit(syn::ExprLit {
                            attrs: Vec::new(),
                            lit: syn::Lit::Str(str_lit),
                        })
                    } else {
                        let block = input.parse::<syn::Block>()?;
                        syn::Expr::Block(syn::ExprBlock {
                            attrs: Vec::new(),
                            label: None,
                            block,
                        })
                    };

                    attr_code.push(quote!( .attr( #attr_css_name , #value )  ));
                }
            } else {
                break;
            }
        }

        // Closing bracket.
        input.parse::<Token![>]>()?;

        // Children.
        let mut children = Vec::new();

        loop {
            // Must be either a static string, block expr or a child tag.
            if let Ok(value) = input.parse::<LitStr>() {
                children.push(quote!( .push(#value) ));
            } else if let Ok(_) = input.parse::<Token![#]>() {
                let block = input.parse::<syn::Block>()?;
                children.push(quote!( .append(#block) ));
            } else if let Ok(block) = input.parse::<syn::Block>() {
                children.push(quote!( .push( #block ) ));
            } else {
                // Check for closing tag.
                let fork = input.fork();

                fork.parse::<Token![<]>()?;
                if fork.parse::<Token![/]>().is_ok() {
                    // Closing delimiter, so assume parent close.
                    break;
                } else {
                    // Assume child tag.
                    let child = Html::parse(input)?;
                    let code = child.code;
                    children.push(quote!( .push( #code ) ));
                }
            }
        }

        // Closing tag.
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let closing_tag = input.parse::<Ident>()?;
        input.parse::<Token![>]>()?;

        if tag != closing_tag {
            panic!("Unclosed <{}> (found </{}>", tag, closing_tag);
        }

        let code = quote!( draco::html::#tag() #(#attr_code)* #(#children)* );

        Ok(Html { code })
    }
}
