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

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

use proc_macro2::TokenStream;


pub fn derive_parser(attr: &syn::Attribute) -> TokenStream {
    quote! {

        pub struct StringCache(RefCell<HashSet<String>>);

        impl StringCache {
            fn new() -> Self {
                StringCache(RefCell::new(HashSet::new()))
            }

            fn intern<'a>(&'a self, s: &str) -> &'a str {
                let mut set = self.0.borrow_mut();
                let mut interned = set.get(s);

                if interned.is_none() {
                    set.insert(s.into());
                    interned = set.get(s);
                }
                unsafe { mem::transmute(interned.unwrap().as_str()) }
            }
        }

        pub struct MPParser<'a> {
            pub lexicon: Box<Lexicon>,
            pub flexicon: StringCache,
            pub math: StringCache,
            pub factstr: StringCache,
            dummy: &'a str,
        }

        #[derive(Parser)]
        #attr
        pub struct FactParser;

        impl<'a> MPParser<'a> {

            pub fn new() -> MPParser<'a> {
                MPParser {
                    lexicon: Box::new(Lexicon::new()),
                    flexicon: StringCache::new(),
                    math: StringCache::new(),
                    factstr: StringCache::new(),
                    dummy: "",
                }
            }

            pub fn parse_text(&'a self, text: &'a str) -> Result<ParseResult<'a>, Error<kparser::Rule>> {
                let parse_tree = kparser::KParser::parse(kparser::Rule::knowledge, text)?.next().expect("initial parse tree");
                let mut facts: Vec<&'a str> = vec![];
                let mut rules: Vec<MPRule> = vec![];
                for pair in parse_tree.into_inner() {
                    match pair.as_rule() {
                        kparser::Rule::fact => {
                            facts.push(pair.as_str());
                        },
                        kparser::Rule::rule => {
                            let mut more_antecedents = VecDeque::new();
                            let mut consequents = vec![];
                            let mut output: Option<&str> = None;
                            for pairset in pair.into_inner() {
                                match pairset.as_rule() {
                                    kparser::Rule::antecedents => {
                                        let mut ant = "";
                                        let mut transforms = "";
                                        let mut conditions = "";
                                        for factpair in pairset.into_inner() {
                                            match factpair.as_rule() {
                                                kparser::Rule::fact => {
                                                    ant = self.factstr.intern(factpair.as_str());
                                                },
                                                kparser::Rule::transforms => {
                                                    transforms = self.math.intern(factpair.as_str());
                                                },
                                                kparser::Rule::conditions => {
                                                    conditions = self.math.intern(factpair.as_str());
                                                },
                                                _ => {}
                                            }
                                        }
                                        more_antecedents.push_back(Antecedents {
                                            fact: Some(ant),
                                            transforms,
                                            conditions,
                                        });
                                    },
                                    kparser::Rule::consequents => {
                                        for factpair in pairset.into_inner() {
                                            match factpair.as_rule() {
                                                kparser::Rule::fact => {
                                                    consequents.push(self.factstr.intern(factpair.as_str()));
                                                },
                                                kparser::Rule::output => {
                                                    output = Some(self.factstr.intern(factpair.as_str()));
                                                },
                                                _ => {}
                                            }
                                        }
                                    },
                                    _ => {}
                                }
                            }
                            let antecedents = more_antecedents.pop_front().unwrap();
                            let rule = MPRule {
                                antecedents,
                                more_antecedents,
                                consequents,
                                matched: HashMap::new(),
                                output,
                            };
                            rules.push(rule);
                        },
                        _ => {}
                    }
                }
                Ok(ParseResult { facts, rules })
            }

            pub fn parse_fact(&'a self, text: &'a str) -> Vec<MPPath<'a>> {
                let parse_tree = FactParser::parse(Rule::fact, text).ok().expect("fact pairset").next().expect("fact pair");
                self.visit_parse_node(parse_tree,
                                      vec![],
                                      vec![],
                                      0)
            }

            fn visit_parse_node(&'a self,
                                parse_tree: Pair<'a, Rule>,
                                root_segments: Vec<&'a MPSegment>,
                                mut all_paths: Vec<MPPath<'a>>,
                                index: usize,
                            ) -> Vec<MPPath> {
                let pretext = parse_tree.as_str();
                let rule = parse_tree.as_rule();
                let name = format!("{:?}", rule);
                let can_be_var = name.starts_with(constants::VAR_RANGE_PREFIX);
                let children: Vec<_> = parse_tree.into_inner().collect();
                let is_leaf = children.len() == 0;
                let text: String;
                if can_be_var || is_leaf {
                    text = format!("{}", pretext);
                } else {
                    text = format!("{}", index);
                }
                let segment = self.lexicon.intern(&name, &text, is_leaf);
                let mut new_root_segments = root_segments.to_vec();
                new_root_segments.push(segment);
                if can_be_var || is_leaf {
                    let segments = new_root_segments.clone();
                    let new_path = MPPath::new(segments);
                    all_paths.push(new_path);
                }
                let mut new_index = 0;
                for child in children {
                    all_paths = self.visit_parse_node(child,
                                                      new_root_segments.clone(),
                                                      all_paths,
                                                      new_index);
                    new_index += 1;
                }
                all_paths
            }
            pub fn substitute_fact(&'a self, fact: Vec<MPPath<'a>>, matching: &'a MPMatching<'a>) -> Vec<MPPath<'a>> {
                if matching.len() == 0 {
                    return fact;
                }
                MPPath::substitute_paths(fact, matching)
            }
            pub fn substitute_fact_fast(&'a self, fact: Vec<MPPath<'a>>, matching: MPMatching<'a>) -> Vec<MPPath<'a>> {
                if matching.len() == 0 {
                    return fact;
                }
                MPPath::substitute_paths_owning(fact, matching)
            }
            pub fn normalize_fact (&'a self, fact: Vec<MPPath<'a>>) -> (MPMatching<'a>, Vec<MPPath<'a>>) {
                let mut varmap: MPMatching<'a> = HashMap::new();
                let mut invarmap: MPMatching<'a> = HashMap::new();
                let mut counter = 1;
                let leaves = fact.as_slice();
                for path in leaves {
                    if path.value.is_empty || !path.value.is_leaf {
                        continue;
                    }
                    if path.value.is_var {
                        let old_var = varmap.get(&path.value);
                        if old_var.is_none() {
                            let new_var = self.lexicon.make_var(counter);
                            counter += 1;
                            varmap.insert(path.value, new_var);
                            invarmap.insert(new_var, path.value);
                        }
                    }
                }
                let new_fact = self.substitute_fact_fast(fact, varmap);
                (invarmap, new_fact)
            }
        }
    }
}
