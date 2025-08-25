extern crate proc_macro;
extern crate syn;

use proc_macro::{TokenStream};
use syn::{parse_macro_input, Expr, ItemFn, Lit, MetaNameValue, Token};
use syn::__private::quote::quote;
use syn::parse::{Parse};

struct TweaksAttribute {
    attrs: Vec<MetaNameValue>,
}

impl Parse for TweaksAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attrs = Vec::new();

        // Parse a comma-separated list of `key = "value"`
        while !input.is_empty() {
            let meta: MetaNameValue = input.parse()?;
            attrs.push(meta);

            // Consume an optional trailing comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(TweaksAttribute { attrs })
    }
}

#[proc_macro_attribute]
pub fn tweak(attrs_orig: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attributes and the item function
    let TweaksAttribute { attrs } = parse_macro_input!(attrs_orig as TweaksAttribute);
    let input_fn = parse_macro_input!(item as ItemFn);

    // Initialize variables to hold parsed values
    let mut name = String::new();
    let mut priority = 100;

    // Iterate through parsed attributes
    for attr in attrs {
        if let (Some(ident), Expr::Lit(expr)) = (attr.path.get_ident(), &attr.value) {
            match (ident.to_string().as_str(), &expr.lit) {
                ("name", Lit::Str(lit_str)) => {
                    name = lit_str.value();
                }
                ("priority", Lit::Int(lit_int)) => {
                    priority = lit_int.base10_parse::<i32>().unwrap_or_else(|_| 100);
                }
                _ => {}
            }
        }
    }

    let func_name = &input_fn.sig.ident;

    let expanded = quote! {
        #input_fn

        inventory::submit! {
            crate::tweaks::tweak::Tweak {
                name: #name,
                priority: #priority,
                execute: |ctx, msg| {
                    #func_name(ctx,msg);
                }
            }
        }
    };

    TokenStream::from(expanded)
}
