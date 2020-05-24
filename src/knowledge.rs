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
            facts: FactSet<'a>,
            rules: RuleSet<'a>,
        }
        impl<'a> KB<'a> {

            pub fn new () -> KB<'a> {
                let mpparser = Box::leak(Box::new(MPParser::new()));
                let root_path = mpparser.lexicon.empty_path();
                Self {
                    mpparser,
                    facts: FactSet::new(),
                    rules: RuleSet::new(root_path),
                }
            }
            fn process_activations(&'a self, mut queues: Queues<'a>) {
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
                            query_rules,
                        } => {
                            queues = self.process_fact(fact, query_rules, queues);
                        },
                        Activation::MPRule {
                            rule, ..
                        } => {
                            self.process_rule(rule);
                        },
                        Activation::Match {
                            rule,
                            matched,
                            query_rules, ..
                        } => {
                            queues = self.process_match(rule, &matched, query_rules, queues);
                        }
                    }
                }
            }
            fn process_rule(&'a self, rule: MPRule<'a>) {
                
                trace!("ADDING RULE {}", rule);
                let MPRule {
                    antecedents,
                    more_antecedents,
                    consequents,
                    matched,
                    output,
                } = rule;
                let n_ants = antecedents.facts.len();
                for n in 0..n_ants {
                    let mut new_ants = vec![];
                    let mut new_ant: Option<&Fact> = None;
                    for (i, ant) in antecedents.facts.iter().enumerate() {
                        if n == i {
                            new_ant = Some(*ant);
                        } else {
                            new_ants.push(*ant);
                        }
                    }
                    let new_conseqs = consequents.clone();
                    let new_more_ants = more_antecedents.clone();
                    let new_rule = MPRule {
                        antecedents: Antecedents {
                            facts: new_ants,
                            transforms: antecedents.transforms,
                            conditions: antecedents.conditions,
                        },
                        more_antecedents: new_more_ants,
                        consequents: new_conseqs,
                        matched: matched.clone(),
                        output,
                    };
                    let (varmap, normal_ant) = self.mpparser.normalize_fact(&new_ant.unwrap());
                    let rule_ref = RuleRef {
                        rule: new_rule,
                        varmap,
                    };
                    let normal_leaf_paths = normal_ant.paths.as_slice();
                    self.rules.follow_and_create_paths(normal_leaf_paths, rule_ref, 1);
                }
            }
            fn process_fact(&'a self,
                            fact: &'a Fact<'a>,
                            query_rules: bool,
                            mut queues: Queues<'a>) -> Queues<'a> {
                
                info!("ADDING FACT: {}", fact);
                let paths = fact.paths.as_slice();
                let response = self.rules.query_paths(paths);
                for (rule_refs, matching) in response {
                    for rule_ref in rule_refs.borrow().iter() {
                        let real_matching = get_real_matching(&matching, &rule_ref.varmap); 
                        queues.match_queue.push_back(Activation::from_matching(rule_ref.rule.clone(), real_matching, query_rules));
                    }
                }
                self.facts.add_fact(&fact);
                queues
            }
            fn process_match(&'a self,
                             mut rule: MPRule<'a>,
                             matching: &MPMatching<'a>,
                             mut query_rules: bool,
                             mut queues: Queues<'a>) -> Queues<'a> {
                let old_len = rule.more_antecedents.len();
                let (nrule, new, passed) = self.preprocess_matched_rule(matching, rule);
                if !passed {
                    return queues;
                }
                rule = nrule;

                if new {
                    if rule.more_antecedents.len() < old_len {
                        query_rules = true;
                    }
                    if query_rules {
                        queues = self.query_rule(&rule, query_rules, queues);
                    }
                    queues.rule_queue.push_back(Activation::from_rule(rule, query_rules));
                } else {
                    for consequent in rule.consequents{
                       let new_consequent = self.mpparser.substitute_fact(consequent, &rule.matched);
                        if !self.facts.ask_fact_bool(&new_consequent) {
                            queues.fact_queue.push_back(Activation::from_fact(new_consequent, query_rules));
                        }
                    }
                    if rule.output.is_some() {
                        let output = self.mpparser.substitute_fact(rule.output.unwrap(), &rule.matched);
                        println!("{}", output.text);
                    }
                }
                queues
            }
            fn query_rule(&'a self,
                          rule: &MPRule<'a>,
                          query_rules: bool,
                          mut queues: Queues<'a>) -> Queues<'a> {

                for i in 0..rule.antecedents.facts.len() {
                    let mut new_ants = rule.antecedents.clone();
                    let ant = new_ants.facts.remove(i);
                    let resps = self.facts.ask_fact(ant);
                    for resp in resps {
                        let new_rule = MPRule {
                            antecedents: new_ants.clone(),
                            more_antecedents: rule.more_antecedents.clone(),
                            consequents: rule.consequents.clone(),
                            matched: rule.matched.clone(),
                            output: rule.output,
                        };
                        queues.match_queue.push_back(Activation::from_matching(new_rule, resp, true));
                    }
                }
                queues
            }
            fn preprocess_matched_rule(&'a self,
                                       matching: &MPMatching<'a>,
                                       rule: MPRule<'a>) -> (MPRule<'a>, bool, bool) {
                let MPRule {
                    mut antecedents,
                    mut more_antecedents,
                    consequents,
                    mut matched,
                    output,
                } = rule;

                matched.extend(matching);

                if antecedents.facts.len() == 0 {
                    if !antecedents.transforms.is_empty() {
                        matched = TParser::process_transforms(antecedents.transforms, matched, &self.mpparser.lexicon);
                    }
                    if !antecedents.conditions.is_empty() {
                        let passed = CParser::check_conditions(antecedents.conditions, &matched, &self.mpparser.lexicon);
                        if !passed {
                            return (MPRule {antecedents, more_antecedents, consequents, matched, output}, false, false);
                        }
                    }

                    if more_antecedents.len() == 0 {
                        return (MPRule {antecedents, more_antecedents, consequents, matched, output}, false, true);
                    } else {
                        antecedents = more_antecedents.pop_front().unwrap();
                    }
                }
                let new_antecedents = antecedents.facts.iter()
                                                 .map(|antecedent| self.mpparser.substitute_fact(antecedent, &matched))
                                                 .collect();
                let mut new_more_antecedents = VecDeque::new();
                while more_antecedents.len() > 0 {
                    let more_ants = more_antecedents.pop_front().unwrap(); 
                    new_more_antecedents.push_back(
                        Antecedents {
                            facts: more_ants.facts.iter()
                                                  .map(|antecedent| self.mpparser.substitute_fact(antecedent, &matched))
                                                  .collect(),
                            transforms: more_ants.transforms,
                            conditions: more_ants.conditions,
                        }
                    );
                }
                let new_consequents = consequents.iter()
                                                 .map(|consequent| self.mpparser.substitute_fact(consequent, &matched))
                                                 .collect();
                (MPRule {
                    antecedents: Antecedents {
                        facts: new_antecedents,
                        transforms: antecedents.transforms,
                        conditions: antecedents.conditions,
                    },
                    more_antecedents: new_more_antecedents,
                    consequents: new_consequents,
                    matched,
                    output,
                }, true, true)

            }
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
                        let act = Activation::from_rule(rule, true);
                        queues.rule_queue.push_back(act);
                    }
                    for fact in facts {
                        if !self.facts.ask_fact_bool(&fact) {
                            let act = Activation::from_fact(fact, false);
                            queues.fact_queue.push_back(act);
                        }
                    }
                }
                self.process_activations(queues);
            }
            fn ask(&'a self, knowledge: &'a str) -> Vec<MPMatching<'a>> {
                let ParseResult { mut facts, .. } = self.mpparser.parse_text(knowledge).ok().expect("parse result");
                let fact = facts.pop().unwrap();
                self.facts.ask_fact(&fact)
            }
        }
    }
}
