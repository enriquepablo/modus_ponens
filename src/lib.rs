#![feature(hash_set_entry)]
#![allow(dead_code)]


pub mod activation;
pub mod constants;
pub mod segment;
pub mod matching;
pub mod path;
pub mod fact;
mod parser;
pub mod facttree;
pub mod ruletree;
mod knowledge;
pub mod lexicon;
pub mod kbase;


extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;


pub fn derive_kbase(input: proc_macro::TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = ast.ident;
    let attr = &ast.attrs[0];

    let derived_parser = parser::derive_parser(attr);
    let derived_kb = knowledge::derive_kb();

    quote! {

        use std::collections::{ HashMap, VecDeque };
        use std::cell::RefCell;

        use log::{info, trace};

        use pest::error::Error;
        use pest::Parser;
        use pest::iterators::Pair;
        use modus_ponens::constants;
        use modus_ponens::activation::{ ParseResult, Activation };
        use modus_ponens::fact::{ Fact, FLexicon };
        use modus_ponens::facttree::FactSet;
        use modus_ponens::kbase::{ KBase, KBGen };
        use modus_ponens::lexicon::Lexicon;
        use modus_ponens::matching::{ MPMatching, get_real_matching_owning };
        use modus_ponens::path::MPPath;
        use modus_ponens::ruletree::{ MPRule, RuleSet, RuleRef };
        use modus_ponens::segment::MPSegment;


        #derived_parser

        #derived_kb

        impl<'a> KBGen<'a> for #name {
            type Output = KB<'a>;
            fn gen_kb() -> KB<'a> {
                KB::new()
            }
        }
    }
}
