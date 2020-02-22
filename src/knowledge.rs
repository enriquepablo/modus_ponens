use std::collections::VecDeque;

use crate::matching::{ SynMatching, get_real_matching };
use crate::fact::Fact;
use crate::factset::FactSet;
use crate::ruletree::{ Rule, RSNode, RuleRef };
use crate::parser::{ parse_text, ParseResult };


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ActType {
    Rule,
    Fact,
    Match,
}

#[derive(Debug)]
pub struct Activation {
    atype: ActType,
    rule: Option<Rule>,
    fact: Option<Fact>,
    matched: Option<SynMatching>,
}

impl Activation {

    pub fn from_fact(fact: Fact) -> Activation {
        Activation {
            atype: ActType::Fact,
            rule: None,
            fact: Some(fact),
            matched: None,
        }
    }
    pub fn from_rule(rule: Rule) -> Activation {
        Activation {
            atype: ActType::Rule,
            rule: Some(rule),
            fact: None,
            matched: None,
        }
    }
    pub fn from_matching(rule: Rule, matched: SynMatching) -> Activation {
        Activation {
            atype: ActType::Match,
            rule: Some(rule),
            fact: None,
            matched: Some(matched),
        }
    }
}

pub struct KnowledgeBase {
    facts: Box<FactSet>,
    rules: Box<RSNode>,
    queue: Box<VecDeque<Activation>>,
}

impl KnowledgeBase {

    pub fn new () -> KnowledgeBase {
        KnowledgeBase {
            facts: Box::new(FactSet::new()),
            rules: Box::new(RSNode::new()),
            queue: Box::new(VecDeque::new()),
        }
    }

    pub fn tell(mut self, knowledge: String) -> KnowledgeBase {
        let ParseResult { rules, facts } = parse_text(&knowledge).ok().unwrap();
        for rule in rules {
            let act = Activation::from_rule(rule);
            self.queue.push_back(act);
        }
        for fact in facts {
            let act = Activation::from_fact(fact);
            self.queue.push_back(act);
        }
        self.process_activations()
    }
    pub fn ask(self, knowledge: String) -> (KnowledgeBase, bool) {
        let ParseResult { rules: _, mut facts } = parse_text(&knowledge).ok().unwrap();
        let fact = facts.pop().unwrap();
        let resps = self.facts.ask_fact(&fact);
        (self, resps.len() > 0)
    }
    fn process_activations(mut self) -> KnowledgeBase {
        while !self.queue.is_empty() {
            let next = self.queue.pop_front().unwrap();
            match next {
                Activation {
                    atype: ActType::Fact,
                    fact: Some(fact), ..
                } => {
                    self = self.process_fact(fact);
                },
                Activation {
                    atype: ActType::Rule,
                    rule: Some(rule), ..
                } => {
                    self = self.process_rule(rule);
                },
                Activation {
                    atype: ActType::Match,
                    rule: Some(rule),
                    fact: None,
                    matched: Some(matched),
                } => {
                    self = self.process_match(rule, matched);
                },
                _ => {}
            }
        }
        self
    }
    fn process_rule(mut self, mut rule: Rule) -> KnowledgeBase {
        let n_ants = rule.antecedents.len();
        for n in 0..n_ants {
            let mut new_ants = vec![];
            let mut new_ant: Option<Fact> = None;
            let Rule { antecedents, consequents } = rule;
            for (i, ant) in antecedents.iter().enumerate() {
                if n == i {
                    new_ant = Some(ant.clone());
                } else {
                    new_ants.push(ant.clone());
                }
            }
            let new_conseqs = consequents.iter().map(|c| c.clone()).collect();
            let new_rule = Rule {
                antecedents: new_ants,
                consequents: new_conseqs
            };
            let (varmap, normal_ant) = new_ant.unwrap().normalize();
            let rule_ref = RuleRef {
                rule: new_rule,
                varmap,
            };
            let zipper = self.rules.zipper(Some(rule_ref));
            self.rules = zipper.follow_and_create_paths(&normal_ant.get_leaf_paths());
            rule = Rule { antecedents, consequents };
        }
        self
    }
    fn process_fact(mut self, fact: Fact) -> KnowledgeBase {
        let izipper = self.rules.izipper();
        let paths = fact.get_leaf_paths();
        let response = izipper.climb(&paths).finish();
        for (rule_refs, matching) in *response {
            for RuleRef { rule, varmap } in rule_refs {
                let real_matching = get_real_matching(&matching, varmap); 
                self.queue.push_back(Activation::from_matching(rule.clone(), real_matching));
            }
        }
        self.facts = Box::new(self.facts.add_fact(fact));
        self
    }
    fn process_match(mut self, rule: Rule, matching: SynMatching) -> KnowledgeBase {
        let Rule { antecedents, consequents } = rule;
        let n_ants = antecedents.len();
        if n_ants > 0 {
            let new_antecedents = antecedents.iter().map(|antecedent| antecedent.substitute(&matching)).collect();
            let new_consequents = consequents.iter().map(|consequent| consequent.substitute(&matching)).collect();
            let new_rule = Rule { antecedents: new_antecedents, consequents: new_consequents };
            self.queue.push_back(Activation::from_rule(new_rule));
        } else {
            for consequent in consequents{
                let new_consequent = consequent.substitute(&matching);
                self.queue.push_back(Activation::from_fact(new_consequent));
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fact_1() {
    }

    #[test]
    fn rule_1() {
    }
}