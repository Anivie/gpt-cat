use quote::{quote, ToTokens};
use syn::LitStr;

pub(crate) struct LitStrWrapper(pub(crate) Vec<LitStr>);

impl ToTokens for LitStrWrapper {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let strings = &self.0;
        tokens.extend(quote! { vec![#( #strings ),*] });
    }
}

pub(crate) struct LitStrWrapper2(pub(crate) Vec<(LitStr, bool)>);
impl ToTokens for LitStrWrapper2 {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let elements = self.0.iter().map(|(lit_str, flag)| {
            quote! {
                (#lit_str, #flag)
            }
        });

        let expanded = quote! {
            vec![#(#elements),*]
        };

        tokens.extend(expanded);
    }
}
