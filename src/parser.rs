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
            pub flexicon: Box<FLexicon<'a>>,
            pub math: StringCache,
            pub factstr: StringCache,
        }

        #[derive(Parser)]
        #attr
        pub struct FactParser;

        impl<'a> MPParser<'a> {

            pub fn new() -> MPParser<'a> {
                MPParser {
                    lexicon: Box::new(Lexicon::new()),
                    flexicon: Box::new(FLexicon::new()),
                    math: StringCache::new(),
                    factstr: StringCache::new(),
                }
            }

            pub fn parse_text(&'a self, text: &'a str) -> Result<ParseResult<'a>, Error<kparser::Rule>> {
                let parse_tree = kparser::KParser::parse(kparser::Rule::knowledge, text)?.next().expect("initial parse tree");
                let mut facts: Vec<&'a Fact> = vec![];
                let mut rules: Vec<MPRule> = vec![];
                for pair in parse_tree.into_inner() {
                    match pair.as_rule() {
                        kparser::Rule::fact => {
                            let (fact, _) = self.parse_fact(pair.as_str(), None);
                            facts.push(fact);
                        },
                        kparser::Rule::rule => {
                            let mut more_antecedents = VecDeque::new();
                            let mut consequents = vec![];
                            let mut antecedents_facts: Vec<&Fact> = vec![];
                            let mut antecedents_transforms: &str = "";
                            let mut antecedents_conditions: &str = "";
                            let mut output: Option<&str> = None;
                            let mut antecedents_count = 0;
                            for pairset in pair.into_inner() {
                                match pairset.as_rule() {
                                    kparser::Rule::antecedents => {
                                        antecedents_count += 1;
                                        let mut pre_ants = vec![];
                                        let mut transforms = "";
                                        let mut conditions = "";
                                        for factpair in pairset.into_inner() {
                                            match factpair.as_rule() {
                                                kparser::Rule::fact => {
                                                    if antecedents_count == 1 {
                                                        let (antecedent, _) = self.parse_fact(factpair.as_str(), None);
                                                        antecedents_facts.push(antecedent);
                                                    } else {
                                                        pre_ants.push(self.factstr.intern(factpair.as_str()));
                                                    }
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
                                        if antecedents_count == 1 {
                                            antecedents_transforms = transforms;
                                            antecedents_conditions = conditions;
                                        } else {
                                            more_antecedents.push_back(PreAntecedents {
                                                facts: pre_ants,
                                                transforms: transforms,
                                                conditions: conditions,
                                            });
                                        }
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
                            let rule = MPRule {
                                antecedents: Antecedents {
                                    facts: antecedents_facts,
                                    transforms: antecedents_transforms,
                                    conditions: antecedents_conditions,
                                },
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

            pub fn parse_fact(&'a self, text: &'a str, matched: Option<MPMatching<'a>>) -> (&'a Fact<'a>, Option<MPMatching<'a>>) {
                let parse_tree = FactParser::parse(Rule::fact, text).ok().expect("fact pairset").next().expect("fact pair");
                self.build_fact(parse_tree, matched)
            }
            
            pub fn build_fact(&'a self, parse_tree: Pair<'a, Rule>, matched: Option<MPMatching<'a>>) -> (&'a Fact<'a>, Option<MPMatching<'a>>) {
                let (all_paths, matched) = self.visit_parse_node(parse_tree,
                                                                 vec![],
                                                                 Box::new(vec![]),
                                                                 0,
                                                                 matched);
                (self.flexicon.from_paths(*all_paths), matched)
            }

            fn visit_parse_node(&'a self,
                                parse_tree: Pair<'a, Rule>,
                                root_segments: Vec<&'a MPSegment>,
                                mut all_paths: Box<Vec<MPPath<'a>>>,
                                index: usize,
                                mut matched: Option<MPMatching<'a>>,
                            ) -> (Box<Vec<MPPath>>, Option<MPMatching<'a>>) {
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
                let mut pending = true;
                if segment.is_var && matched.is_some() {
                    let matching = matched.unwrap();
                    let value = matching.get(segment);
                    if value.is_some() {
                        let val = value.unwrap();
                        let parse_tree = FactParser::parse(Rule::var_range, &val.text).ok().unwrap().next().expect("a matched production");
                        let (new_all_paths, _) = self.visit_parse_node(parse_tree,
                                                                       root_segments.clone(),
                                                                       all_paths,
                                                                       0,
                                                                       None);
                        all_paths = new_all_paths;
                        pending = false;
                    }
                    matched = Some(matching);
                }
                if pending {
                    let mut new_root_segments = root_segments.to_vec();
                    new_root_segments.push(segment);
                    if can_be_var || is_leaf {
                        let segments = new_root_segments.clone();
                        let new_path = MPPath::new(segments);
                        all_paths.push(new_path);
                    }
                    let mut new_index = 0;
                    for child in children {
                        let (new_all_paths, old_matched) = self.visit_parse_node(child,
                                                                                 new_root_segments.clone(),
                                                                                 all_paths,
                                                                                 new_index,
                                                                                 matched);
                        matched = old_matched;
                        all_paths = new_all_paths;
                        new_index += 1;
                    }
                }
                (all_paths, matched)
            }
            pub fn substitute_fact(&'a self, fact: &'a Fact<'a>, matching: &MPMatching<'a>) -> &'a Fact<'a> {
                let new_paths = MPPath::substitute_paths(&fact.paths, matching);
                let text = new_paths.iter()
                                    .map(|path| path.value.text.as_str())
                                    .collect::<Vec<&str>>()
                                    .concat();

                // XXX LEAK!
                let stext = Box::leak(text.into_boxed_str());
                
                let parse_tree = FactParser::parse(Rule::fact, stext).ok().unwrap().next().expect("2nd fact pair");
                let all_paths = Box::new(Vec::with_capacity(fact.paths.len()));
                let (all_paths, _) = self.visit_parse_node(parse_tree,
                                                               vec![],
                                                               all_paths,
                                                               0,
                                                               None);
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
