extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

use proc_macro2::TokenStream;


pub fn derive_kb() -> TokenStream {
    quote! {
        pub struct KB<'a> {
            mpparser: &'a MPParser<'a>,
            facts: FactSet<'a>,
            rules: RuleSet<'a>,
            queue: RefCell<VecDeque<Activation<'a>>>,
        }

        impl<'a> KB<'a> {

            pub fn new () -> KB<'a> {
                let mpparser = Box::leak(Box::new(MPParser::new()));
                let root_path = mpparser.lexicon.empty_path();
                Self {
                    mpparser,
                    facts: FactSet::new(),
                    rules: RuleSet::new(root_path),
                    queue: RefCell::new(VecDeque::new()),
                }
            }



            fn process_activations(&'a self) {
                while !self.queue.borrow().is_empty() {
                    let next = self.queue.borrow_mut().pop_front().unwrap();
                    match next {
                        Activation::Fact {
                            fact,
                            query_rules,
                        } => {
                            self.process_fact(fact, query_rules);
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
                            self.process_match(rule, &matched, query_rules);
                        }
                    }
                }
            }
            fn process_rule(&'a self, rule: MPRule<'a>) {
                
                trace!("ADDING RULE {}", rule);
                let MPRule {
                    antecedents,
                    more_antecedents,
                    consequents
                } = rule;
                let n_ants = antecedents.len();
                for n in 0..n_ants {
                    let mut new_ants = vec![];
                    let mut new_ant: Option<&Fact> = None;
                    for (i, ant) in antecedents.iter().enumerate() {
                        if n == i {
                            new_ant = Some(*ant);
                        } else {
                            new_ants.push(*ant);
                        }
                    }
                    let new_conseqs = consequents.clone();
                    let new_more_ants = more_antecedents.clone();
                    let new_rule = MPRule {
                        antecedents: new_ants,
                        more_antecedents: new_more_ants,
                        consequents: new_conseqs
                    };
                    let (varmap, normal_ant) = self.mpparser.normalize_fact(&new_ant.unwrap());
                    let rule_ref = RuleRef {
                        rule: new_rule,
                        varmap,
                    };
                    let normal_leaf_paths = normal_ant.paths.as_slice();
                    self.rules.follow_and_create_paths(normal_leaf_paths, rule_ref, 1);
                }
                MPRule {
                    antecedents,
                    more_antecedents,
                    consequents
                };
            }
            fn process_fact(&'a self,
                            fact: &'a Fact<'a>,
                            query_rules: bool) {
                
                info!("ADDING FACT: {}", fact);
                let paths = fact.paths.as_slice();
                let response = self.rules.query_paths(paths);
                for (rule_refs, matching) in response {
                    for rule_ref in rule_refs {
                        let real_matching = get_real_matching_owning(matching.clone(), rule_ref.varmap.clone()); 
                        self.queue.borrow_mut().push_back(Activation::from_matching(rule_ref.rule.clone(), real_matching, query_rules));
                    }
                }
                self.facts.add_fact(&fact);
            }
            fn process_match(&'a self,
                             mut rule: MPRule<'a>,
                             matching: &MPMatching<'a>,
                             mut query_rules: bool) {
                let old_len = rule.more_antecedents.len();
                let (nrule, new) = self.preprocess_matched_rule(matching, rule);
                rule = nrule;

                if new {
                    if rule.more_antecedents.len() < old_len {
                        query_rules = true;
                    }
                    if query_rules {
                        let nrule = self.query_rule(rule, query_rules);
                        rule = nrule;
                    }
                    self.queue.borrow_mut().push_back(Activation::from_rule(rule, query_rules));
                } else {
                   for consequent in rule.consequents{
                       let new_consequent = self.mpparser.substitute_fact(&consequent, matching);
                        if !self.facts.ask_fact_bool(&new_consequent) {
                            self.queue.borrow_mut().push_back(Activation::from_fact(new_consequent, query_rules));
                        }
                   }
                }
            }
            fn query_rule(&'a self,
                          rule: MPRule<'a>,
                          query_rules: bool) -> MPRule {

                for i in 0..rule.antecedents.len() {
                    let mut new_ants = rule.antecedents.clone();
                    let ant = new_ants.remove(i);
                    let resps = self.facts.ask_fact(ant);
                    for resp in resps {
                        let new_rule = MPRule {
                            antecedents: new_ants.clone(),
                            more_antecedents: rule.more_antecedents.clone(),
                            consequents: rule.consequents.clone(),
                        };
                        self.process_match(new_rule, &resp, query_rules);
                    }
                }
                rule
            }
            fn preprocess_matched_rule(&'a self,
                                       matching: &MPMatching<'a>,
                                       rule: MPRule<'a>) -> (MPRule<'a>, bool) {
                let MPRule {
                    mut antecedents,
                    mut more_antecedents,
                    consequents
                } = rule;
                if antecedents.len() == 0 {
                    if more_antecedents.len() == 0 {
                        return (MPRule {antecedents, more_antecedents, consequents}, false);
                    } else {
                        antecedents = more_antecedents.remove(0);
                    }
                }
                let new_antecedents = antecedents.iter()
                                                 .map(|antecedent| self.mpparser.substitute_fact(antecedent, matching))
                                                 .collect();
                let mut new_more_antecedents = Vec::new();
                while more_antecedents.len() > 0 {
                    let more_ants = more_antecedents.remove(0); 
                    new_more_antecedents.push(more_ants.iter()
                                                       .map(|antecedent| self.mpparser.substitute_fact(antecedent, matching))
                                                       .collect());
                }
                let new_consequents = consequents.iter()
                                                 .map(|consequent| self.mpparser.substitute_fact(consequent, matching))
                                                 .collect();
                (MPRule {
                    antecedents: new_antecedents,
                    more_antecedents: new_more_antecedents,
                    consequents: new_consequents
                }, true)

            }
        }
        impl<'a> KBase<'a> for KB<'a> {
            fn tell(&'a self, knowledge: &'a str) {
                let result = self.mpparser.parse_text(knowledge);
                if result.is_err() {
                    panic!("Parsing problem! {}", result.err().unwrap());
                }
                let ParseResult { rules, facts } = result.ok().unwrap();
                for rule in rules {
                    let act = Activation::from_rule(rule, true);
                    self.queue.borrow_mut().push_back(act);
                    self.process_activations();
                }
                for fact in facts {
                    if !self.facts.ask_fact_bool(&fact) {
                        let act = Activation::from_fact(fact, false);
                        self.queue.borrow_mut().push_back(act);
                        self.process_activations();
                    }
                }
            }
            fn ask(&'a self, knowledge: &'a str) -> bool {
                let ParseResult { mut facts, .. } = self.mpparser.parse_text(knowledge).ok().unwrap();
                let fact = facts.pop().unwrap();
                let resps = self.facts.ask_fact(&fact);
                resps.len() > 0
            }
        }
    }
}
