extern crate modus_ponens;
extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(KBGen, attributes(grammar, grammar_inline))]
pub fn derive_parser(input: TokenStream) -> TokenStream {
    modus_ponens::derive_kbase(input.into()).into()
}
