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

        #[derive(Parser)]
        #attr
        pub struct MPParser<'a> {
            pub lexicon: Box<Lexicon>,
            pub flexicon: Box<FLexicon<'a>>,
        }

        impl<'a> MPParser<'a> {

            pub fn new() -> MPParser<'a> {
                MPParser {
                    lexicon: Box::new(Lexicon::new()),
                    flexicon: Box::new(FLexicon::new()),
                }
            }

            pub fn parse_text(&'a self, text: &'a str) -> Result<ParseResult<'a>, Error<Rule>> {
                let parse_tree = MPParser::parse(Rule::knowledge, text)?.next().unwrap();
                let mut facts: Vec<&'a Fact> = vec![];
                let mut rules: Vec<MPRule> = vec![];
                for pair in parse_tree.into_inner() {
                    match pair.as_rule() {
                        Rule::fact => {
                            let fact = self.build_fact(pair);
                            facts.push(fact);
                        },
                        Rule::rule => {
                            let mut more_antecedents = Vec::new();
                            let mut consequents = vec![];
                            for pairset in pair.into_inner() {
                                match pairset.as_rule() {
                                    Rule::antecedents => {
                                        let mut ants = vec![];
                                        for factpair in pairset.into_inner() {
                                            match factpair.as_rule() {
                                                Rule::fact => {
                                                    let antecedent = self.build_fact(factpair);
                                                    ants.push(antecedent);
                                                },
                                                _ => {}
                                            }
                                        }
                                        more_antecedents.push(ants);
                                    },
                                    Rule::consequents => {
                                        for factpair in pairset.into_inner() {
                                            match factpair.as_rule() {
                                                Rule::fact => {
                                                    let consequent = self.build_fact(factpair);
                                                    consequents.push(consequent);
                                                },
                                                _ => {}
                                            }
                                        }
                                    },
                                    _ => {}
                                }
                            }
                            let antecedents = more_antecedents.remove(0);
                            let rule = MPRule {
                                antecedents,
                                more_antecedents,
                                consequents
                            };
                            rules.push(rule);
                        },
                        _ => {}
                    }
                }
                Ok(ParseResult { facts, rules })
            }

            pub fn parse_fact(&'a self, text: &'a str) -> &'a Fact<'a> {
                let parse_tree = MPParser::parse(Rule::fact, text).ok().unwrap().next().unwrap();
                self.build_fact(parse_tree)
            }
            
            pub fn build_fact(&'a self, parse_tree: Pair<'a, Rule>) -> &'a Fact<'a> {
                let all_paths = self.visit_parse_node(parse_tree,
                                                      vec![],
                                                      Box::new(vec![]),
                                                      0);
                self.flexicon.from_paths(*all_paths)
            }

            fn visit_parse_node(&'a self,
                                parse_tree: Pair<'a, Rule>,
                                root_segments: Vec<&'a MPSegment>,
                                mut all_paths: Box<Vec<MPPath<'a>>>,
                                index: usize,
                            ) -> Box<Vec<MPPath>> {
                let pretext = parse_tree.as_str();
                let rule = parse_tree.as_rule();
                let name = format!("{:?}", rule);
                let can_be_var = name.starts_with(constants::VAR_RANGE_PREFIX);
                let children: Vec<_> = parse_tree.into_inner().collect();
                let is_leaf = children.len() == 0;
                let text;
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
            pub fn substitute_fact(&'a self, fact: &'a Fact<'a>, matching: &MPMatching<'a>) -> &'a Fact<'a> {
                let new_paths = MPPath::substitute_paths(&fact.paths, matching);
                let text = new_paths.iter()
                                    .map(|path| path.value.text.as_str())
                                    .collect::<Vec<&str>>()
                                    .concat();

                // XXX LEAK!
                let stext = Box::leak(text.into_boxed_str());
                
                let parse_tree = MPParser::parse(Rule::fact, stext).ok().unwrap().next().unwrap();
                let all_paths = Box::new(Vec::with_capacity(fact.paths.len()));
                let all_paths = self.visit_parse_node(parse_tree,
                                                                         vec![],
                                                                         all_paths,
                                                                         0);
                self.flexicon.from_paths_and_string(stext, *all_paths)
            }
            pub fn substitute_fact_fast(&'a self, fact: &'a Fact, matching: MPMatching<'a>) -> &'a Fact<'a> {
                let new_paths = MPPath::substitute_paths_owning(&fact.paths, matching);
                self.flexicon.from_paths(new_paths)
            }
            pub fn normalize_fact (&'a self, fact: &'a Fact<'a>) -> (MPMatching<'a>, &'a Fact<'a>) {
                let mut varmap: MPMatching<'a> = HashMap::new();
                let mut invarmap: MPMatching<'a> = HashMap::new();
                let mut counter = 1;
                let leaves = fact.paths.as_slice();
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
