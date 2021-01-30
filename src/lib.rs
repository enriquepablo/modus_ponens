// Copyright (c) 2020 by Enrique Pérez Arnaud <enrique at cazalla.net>    
//    
// This file is part of the modus_ponens project.    
// http://www.modus_ponens.net    
//    
// The modus_ponens project is free software: you can redistribute it and/or modify    
// it under the terms of the GNU General Public License as published by    
// the Free Software Foundation, either version 3 of the License, or    
// (at your option) any later version.    
//    
// The modus_ponens project is distributed in the hope that it will be useful,    
// but WITHOUT ANY WARRANTY; without even the implied warranty of    
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the    
// GNU General Public License for more details.    
//    
// You should have received a copy of the GNU General Public License    
// along with any part of the modus_ponens project.    
// If not, see <http://www.gnu.org/licenses/>.
//!
//! This software helps dealing with data, in whatever form or shape.
//! It allows you to keep your data in knowledge bases under any shape
//! and structural detail you may feel appropriate,
//! and to query and massage it at any level of the detail you may have bothered to specify.
//!
//! You describe the form of your data in a Parsing Expression Grammar (PEG),
//! so that your data can be contained in a set of sentences compliant with this grammar.
//!
//! In the grammar, you specify which of the productions needed to build up sentences
//! you may want to abstract away in the rules of your knoledge bases, and in queries -
//! i.e., which grammatic (syntactic) components correspond to logically quantifiable symbols.
//!
//! Then modus_ponens provides an inference engine that allows you to deal with
//! knowledge bases compliant with the provided PEG,
//! in which adding new sentences and rules is,
//! in terms of algorithmic complexity, 
//! independent of the number of sentences and rules in the knowledge base,
//! and only dependent on the number of consequences of the added knowledge.
//!

#![feature(hash_set_entry)]
#![allow(dead_code)]


pub mod activation;
pub mod constants;
pub mod segment;
pub mod matching;
pub mod path;
//pub mod fact;
mod parser;
pub mod facttree;
pub mod ruletree;
mod knowledge;
pub mod lexicon;
pub mod kbase;
pub mod kparser;
pub mod transform;
pub mod transform_num;
pub mod transform_str;
pub mod condition;


extern crate pest;
#[macro_use]
extern crate pest_derive;

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

        use std::collections::{ HashMap, HashSet, VecDeque };
        use std::cell::RefCell;
        use std::mem;

        use log::{info, trace};

        use pest::error::Error;
        use pest::Parser;
        use pest::iterators::Pair;
        use modus_ponens::constants;
        use modus_ponens::activation::{ ParseResult, Activation };
        use modus_ponens::facttree::FactSet;
        use modus_ponens::kbase::{ KBase, KBGen };
        use modus_ponens::lexicon::Lexicon;
        use modus_ponens::matching::{ MPMatching, get_real_matching};
        use modus_ponens::path::MPPath;
        use modus_ponens::ruletree::{ Antecedents, MPRule, RuleSet, RuleRef };
        use modus_ponens::segment::MPSegment;
        use modus_ponens::kparser;
        use modus_ponens::transform::TParser;
        use modus_ponens::condition::CParser;


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
