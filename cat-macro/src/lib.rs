#![feature(proc_macro_diagnostic)]
#![cfg_attr(debug_assertions, allow(warnings))]

mod type_wrapper;
mod new_keyword;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::token::Paren;
use syn::{bracketed, parenthesized, parse_macro_input, LitStr, Token};
use crate::new_keyword::keyword;
use crate::type_wrapper::{LitStrWrapper, LitStrWrapper2};

struct Param {
    names: Vec<LitStr>,
    help: LitStr,
    example: Option<LitStr>,

    param: Option<Vec<(LitStr, bool)>>,
    param_description: Option<Vec<LitStr>>,
}

impl Parse for Param {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let names = {
            let names_in_bracketed: ParseBuffer = {
                let content;
                bracketed!(content in input);
                content
            };
            let mut names = vec![];
            loop {
                let name = names_in_bracketed.parse::<LitStr>()?;
                names.push(name);

                if !names_in_bracketed.is_empty() {
                    names_in_bracketed.parse::<Token![|]>()?;
                } else {
                    break;
                }
            }
            names
        };

        let help = {
            input.parse::<keyword::help>()
                .map_err(|x| syn::Error::new(x.span(), "Every command need a help message."))?;
            input.parse::<LitStr>()?
        };

        let example = {
            if input.peek(keyword::example) {
                input.parse::<keyword::example>()?;
                let example = input.parse::<LitStr>()?;
                Some(example)
            } else {
                None
            }
        };

        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
        }else {
            return Ok(Param {
                names,
                help,
                example,
                param: None,
                param_description: None,
            });
        }

        let mut args = vec![];
        let mut describe = vec![];
        loop {
            if input.peek(Paren) {
                let arg_name = {
                    let content;
                    parenthesized!(content in input);
                    content.parse::<LitStr>()?
                };
                input.parse::<Token![=>]>()?;
                let arg_help = input.parse::<LitStr>()?;
                args.push((arg_name, false));
                describe.push(arg_help);
                input.parse::<Token![,]>()?;
            }else {
                let arg_name = input.parse::<LitStr>()?;
                input.parse::<Token![=>]>()?;
                let arg_help = input.parse::<LitStr>()?;
                args.push((arg_name, true));
                describe.push(arg_help);
                input.parse::<Token![,]>()?;
            }

            if input.is_empty() { break; }
        }

        Ok(Param {
            names,
            help,
            example,
            param: Some(args),
            param_description: Some(describe),
        })
    }
}

fn expand_to_tokens<T: ToTokens>(input: &Option<T>) -> proc_macro2::TokenStream {
    match input {
        Some(value) => quote!(Some(#value)),
        None => quote!(None)
    }
}

#[proc_macro]
pub fn describe(input: TokenStream) -> TokenStream {
    let Param {
        names,
        help,
        example,
        param,
        param_description,
    } = parse_macro_input!(input as Param);
    let example = expand_to_tokens(&example);
    let param = match param {
        None => quote!(None),
        Some(param) => {
            let param = LitStrWrapper2(param);
            quote!(Some(#param))
        }
    };
    let param_description = match param_description {
        None => quote!(None),
        Some(param_description) => {
            let param_description = LitStrWrapper(param_description);
            quote!(Some(#param_description))
        }
    };

    let output = quote! {
        CommandDescription {
            name: vec![#(#names),*],
            help: #help,
            example: #example,
            param: #param,
            param_description: #param_description,
        }
    };

    TokenStream::from(output)
}