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


pub fn derive_kb() -> TokenStream {
    quote! {

        pub struct Queues<'a> {
            rule_queue: VecDeque<Activation<'a>>,
            match_queue: VecDeque<Activation<'a>>,
            fact_queue: VecDeque<Activation<'a>>,
        }
        impl<'a> Queues<'a> {

            pub fn new () -> Queues<'a> {
                Self {
                    rule_queue: VecDeque::new(),
                    match_queue: VecDeque::new(),
                    fact_queue: VecDeque::new(),
                }
            }
        }

        pub struct KB<'a> {
            mpparser: &'a MPParser<'a>,
            tparser: TParser<'a>,
            facts: FactSet<'a>,
            rules: RuleSet<'a>,
        }
        impl<'a> KBase<'a> for KB<'a> {
            fn tell(&'a self, knowledge: &'a str) {
                let result = self.mpparser.parse_text(knowledge.trim());
                let mut queues = Queues::new();
                if result.is_err() {
                    panic!("Parsing problem! {}", result.err().unwrap());
                } else {
                    let ParseResult { rules, facts } = result.ok().unwrap();
                    for rule in rules {
                        let act = Activation::from_rule(rule, None, true);
                        queues.rule_queue.push_back(act);
                        queues = self.process_activations(queues);
                    }
                    for fact in facts {
                        let act = Activation::from_fact(fact, None, false);
                        queues.fact_queue.push_back(act);
                        queues = self.process_activations(queues);
                    }
                }
            }
            fn ask(&'a self, knowledge: &'a str) -> Vec<MPMatching<'a>> {
                let ParseResult { mut facts, .. } = self.mpparser.parse_text(knowledge).ok().expect("parse result");
                let fact = facts.pop().unwrap();
                let q = self.mpparser.parse_fact(fact);
                let (resp, _, _) = self.facts.ask_fact(q);
                resp
            }
        }
        impl<'a> KB<'a> {

            pub fn new () -> KB<'a> {
                let mpparser = Box::leak(Box::new(MPParser::new()));
                let root_path = mpparser.lexicon.empty_path();
                let tparser = TParser::new(&mpparser.lexicon);
                Self {
                    mpparser,
                    tparser,
                    facts: FactSet::new(),
                    rules: RuleSet::new(root_path),
                }
            }
            fn process_activations(&'a self, mut queues: Queues<'a>) -> Queues<'a> {
                loop {
                    let mut next_opt = queues.rule_queue.pop_front();
                    if next_opt.is_none() {
                        next_opt = queues.match_queue.pop_front();
                        if next_opt.is_none() {
                            next_opt = queues.fact_queue.pop_front();
                            if next_opt.is_none() {
                                break
                            }
                        }
                    }
                    match next_opt.unwrap() {
                        Activation::Fact {
                                fact,
                                matched,
                                query_rules,
                        } => {
                            queues = self.process_fact(fact, matched, query_rules, queues);
                        },
                        Activation::MPRule {
                            rule,
                            paths,
                            query_rules,
                        } => {
                            queues = self.process_rule(rule, paths, query_rules, queues);
                        },
                        Activation::Match {
                            rule,
                            matched,
                            query_rules,
                        } => {
                            queues = self.process_match(rule, matched, query_rules, queues);
                        },
                    }
                }
                queues
            }
            fn process_rule(&'a self, mut rule: MPRule<'a>, paths: Option<Vec<MPPath<'a>>>, query_rules: bool, mut queues: Queues<'a>) -> Queues<'a> {
                
                trace!("ADDING RULE {}", rule);

                if rule.antecedents.fact.is_some() {
                    if constants::NEW_RULES && paths.is_none() && query_rules {
                        let (new_queues, new_paths, old_rule, unique) = self.query_rule(rule, queues);
                        rule = old_rule;
                        queues = new_queues;
                    };
                    let MPRule {
                        antecedents: Antecedents {
                            fact,
                            transforms,
                            conditions,
                        },
                        more_antecedents,
                        consequents,
                        mut matched,
                        output,
                    } = rule;
                    let ant = fact.unwrap();

                    let mut new_antecedent: Vec<MPPath<'a>>;
                    if paths.is_none() {
                        new_antecedent = self.mpparser.parse_fact(ant);
                        if matched.len() > 0 {
                            let (new_ant, old_matched, _) = self.mpparser.substitute_fact(new_antecedent, matched);
                            new_antecedent = new_ant;
                            matched = old_matched;
                        }
                    } else {
                        new_antecedent = paths.unwrap();
                    }
                    let (varmap, normal_ant) = self.mpparser.normalize_fact(new_antecedent);

                    let rule = MPRule {
                        antecedents: Antecedents {
                            fact: None,
                            transforms,
                            conditions,
                        },
                        more_antecedents,
                        consequents,
                        matched,
                        output,
                    };
                    let rule_ref = RuleRef {
                        rule: rule,
                        varmap,
                    };
                    self.rules.follow_and_create_paths(normal_ant, rule_ref, 1);
                } else {
                    queues.match_queue.push_back(Activation::from_matching(rule, None, query_rules));
                }
                queues
            }
            fn process_fact(&'a self,
                            fact: &'a str,
                            matching: Option<MPMatching<'a>>,
                            query_rules: bool,
                            mut queues: Queues<'a>) -> Queues<'a> {


                let mut fact_paths = self.mpparser.parse_fact(fact);
                let mut fact_string: Option<String> = None;
                if matching.is_some() {
                    let (new_fact_paths, _, fact_str) = self.mpparser.substitute_fact(fact_paths, matching.unwrap());
                    fact_paths = new_fact_paths;
                    fact_string = fact_str;
                }
                let (exists, paths) = self.facts.ask_fact_bool(fact_paths);
                fact_paths = paths;
                if  exists {
                    return queues;
                }
                let (response, paths) = self.rules.query_paths(fact_paths);
                fact_paths = paths;
                for (rule_refs, matching) in response {
                    for rule_ref in rule_refs.borrow().iter() {
                        let real_matching = get_real_matching(&matching, &rule_ref.varmap); 
                        queues.match_queue.push_back(Activation::from_matching(rule_ref.rule.clone(), Some(real_matching), query_rules));
                    }
                }
                if fact_string.is_some() {
                    info!("ADDING FACT: {}", &fact_string.unwrap());
                } else {
                    info!("ADDING FACT: {}", &fact);
                }
                self.facts.add_fact(fact_paths);
                queues
            }
            fn process_match(&'a self,
                             mut rule: MPRule<'a>,
                             matching: Option<MPMatching<'a>>,
                             mut query_rules: bool,
                             mut queues: Queues<'a>) -> Queues<'a> {
                let old_len = rule.more_antecedents.len();
                let (nrule, new, passed, _) = self.preprocess_matched_rule(rule, matching);
                if !passed {
                    return queues;
                }
                rule = nrule;

                if new {
                    if rule.more_antecedents.len() < old_len {
                        query_rules = true;
                    }
                    let mut paths: Option<Vec<MPPath>> = None;
                    let mut add_rule = true;
                    if query_rules {
                        let (new_queues, new_paths, old_rule, unique) = self.query_rule(rule, queues);
                        paths = new_paths;
                        queues = new_queues;
                        rule = old_rule;
                        add_rule = !unique;
                    }
                    if add_rule {
                        queues.rule_queue.push_back(Activation::from_rule(rule, paths, query_rules));
                    }
                } else {
                    for consequent in rule.consequents{
                        queues.fact_queue.push_back(Activation::from_fact(consequent, Some(rule.matched.clone()), query_rules));
                    }
                    if rule.output.is_some() {
                        let pre_output = self.mpparser.parse_fact(rule.output.unwrap());
                        let (_, _, output) = self.mpparser.substitute_fact(pre_output, rule.matched);
                        if output.is_some() {
                            println!("ADDING FACT: {}", &output.unwrap());
                        } else {
                            println!("ADDING FACT: {}", rule.output.as_ref().unwrap());
                        }
                    }
                }
                queues
            }
            fn query_rule(&'a self,
                          mut rule: MPRule<'a>,
                          mut queues: Queues<'a>) -> (Queues<'a>, Option<Vec<MPPath<'a>>>, MPRule<'a>, bool) {

                let mut paths: Option<Vec<MPPath>> = None;
                let mut unique = false;
                if rule.antecedents.fact.is_some() {
                    let MPRule {
                        mut antecedents,
                        mut more_antecedents,
                        consequents,
                        mut matched,
                        output,
                    } = rule;
                    let fact_str = antecedents.fact.as_ref().unwrap();
                    let mut pre_ant = self.mpparser.parse_fact(fact_str);
                    if matched.len() > 0 {
                        let (new_pre_ant, old_matched, _) = self.mpparser.substitute_fact(pre_ant, matched);
                        pre_ant = new_pre_ant;
                        matched = old_matched;
                    }
                    rule = MPRule {
                        antecedents,
                        more_antecedents,
                        consequents,
                        matched,
                        output,
                    };
                    let (resps, old_paths, new_unique) = self.facts.ask_fact(pre_ant);
                    unique = new_unique;
                    paths = Some(old_paths);
                    for resp in resps {
                        let mut new_rule = rule.clone();
                        new_rule.antecedents.fact = None;
                        queues.match_queue.push_back(Activation::from_matching(new_rule, Some(resp), true));
                    }
                }
                (queues, paths, rule, unique)
            }
            fn preprocess_matched_rule(&'a self,
                                       rule: MPRule<'a>,
                                       mut matching: Option<MPMatching<'a>>) -> (MPRule<'a>, bool, bool, Option<MPMatching<'a>>) {
                let MPRule {
                    mut antecedents,
                    mut more_antecedents,
                    consequents,
                    mut matched,
                    output,
                } = rule;

                if matching.is_some() {
                    let m = matching.unwrap();
                    matched.extend(&m);
                    matching = Some(m);
                }
                let Antecedents { fact, transforms, conditions } = antecedents;

                if !transforms.is_empty() {
                    matched = self.tparser.process_transforms(transforms, matched);
                }
                if !conditions.is_empty() {
                    let passed = CParser::check_conditions(conditions, &matched, &self.mpparser.lexicon);
                    if !passed {
                        return (MPRule {antecedents: Antecedents { fact, transforms, conditions }, more_antecedents, consequents, matched, output}, false, false, matching);
                    }
                }

                if more_antecedents.len() == 0 {
                    return (MPRule {antecedents: Antecedents { fact, transforms, conditions }, more_antecedents, consequents, matched, output}, false, true, matching);
                } else {
                    antecedents = more_antecedents.pop_front().unwrap();
                }

                (MPRule {
                    antecedents,
                    more_antecedents,
                    consequents,
                    matched,
                    output,
                }, true, true, matching)

            }
        }


    }
}
