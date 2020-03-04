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
    facts: FactSet,
    rules: Box<RSNode>,
    queue: Box<VecDeque<Activation>>,
}

impl KnowledgeBase {

    pub fn new () -> KnowledgeBase {
        KnowledgeBase {
            facts: FactSet::new(),
            rules: Box::new(RSNode::new()),
            queue: Box::new(VecDeque::new()),
        }
    }

    pub fn tell(mut self, knowledge: &str) -> KnowledgeBase {
        let result = parse_text(knowledge);
        if result.is_err() {
            panic!("Parsing problem! {}", result.err().unwrap());
        }
        let ParseResult { rules, facts } = result.ok().unwrap();
        for rule in rules {
            let act = Activation::from_rule(rule);
            self.queue.push_back(act);
            self = self.process_activations();
        }
        for fact in facts {
            let act = Activation::from_fact(fact);
            self.queue.push_back(act);
            self = self.process_activations();
        }
        self
    }
    pub fn ask(&self, knowledge: &str) -> bool {
        let ParseResult { rules: _, mut facts } = parse_text(knowledge).ok().unwrap();
        let fact = facts.pop().unwrap();
        let resps = self.facts.ask_fact(&fact);
        resps.len() > 0
    }
    pub fn check_fact(&self, fact: &Fact) -> bool {
        let resps = self.facts.ask_fact(fact);
        resps.len() > 0
    }
    fn process_activations(mut self) -> Self {
        while !self.queue.is_empty() {
            let next = self.queue.pop_front().unwrap();
            match next {
                Activation {
                    atype: ActType::Fact,
                    fact: Some(fact), ..
                } => {
                    if !self.check_fact(&fact) {
                        self = self.process_fact(fact);
                    }
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
                    matched: Some(matched), ..
                } => {
                    self = self.process_match(rule, matched);
                },
                _ => {}
            }
        }
        self
    }
    fn process_rule(mut self, mut rule: Rule) -> Self {
        
        //println!("ADDING RULE: {}", rule);
        let n_ants = rule.antecedents.len();
        for n in 0..n_ants {
            let mut new_ants = vec![];
            let mut new_ant: Option<Fact> = None;
            let Rule {
                antecedents,
                more_antecedents,
                consequents
            } = rule;
            for (i, ant) in antecedents.iter().enumerate() {
                if n == i {
                    new_ant = Some(ant.clone());
                } else {
                    new_ants.push(ant.clone());
                }
            }
            let new_conseqs = consequents.clone();
            let new_more_ants = more_antecedents.clone();
            let new_rule = Rule {
                antecedents: new_ants,
                more_antecedents: new_more_ants,
                consequents: new_conseqs
            };
            let (varmap, normal_ant) = new_ant.unwrap().normalize();
            let rule_ref = RuleRef {
                rule: new_rule,
                varmap,
            };
            let zipper = self.rules.zipper(Some(rule_ref));
            self.rules = zipper.follow_and_create_paths(&normal_ant.get_leaf_paths());
            rule = Rule {
                antecedents,
                more_antecedents,
                consequents
            };
        }
        self
    }
    fn process_fact(mut self, fact: Fact) -> Self {
        
        println!("ADDING FACT: {}", fact);
        
        let izipper = self.rules.izipper();
        let paths = fact.get_leaf_paths();
        let response = izipper.climb(&paths).finish();
        for (rule_refs, matching) in *response {
            for RuleRef { rule, varmap } in rule_refs {
                let real_matching = get_real_matching(&matching, varmap); 
                self.queue.push_back(Activation::from_matching(rule.clone(), real_matching));
            }
        }
        self.facts = self.facts.add_fact(fact);
        self
    }
    fn process_match(mut self, rule: Rule, matching: SynMatching) -> Self {
        let Rule {
            antecedents,
            mut more_antecedents,
            consequents
        } = rule;
        let n_ants = antecedents.len();
        if n_ants > 0 {
            let new_rule = self.preprocess_matched_rule(matching,
                                                        antecedents,
                                                        more_antecedents,
                                                        consequents);
            self.queue.push_back(Activation::from_rule(new_rule));
        } else {
            if more_antecedents.len() > 0 {
                let new_ants = more_antecedents.pop_front().unwrap();
                let new_rule = self.preprocess_matched_rule(matching,
                                                            new_ants,
                                                            more_antecedents,
                                                            consequents);
                self.queue.push_back(Activation::from_rule(new_rule));
            } else {
                for consequent in consequents{
                    let new_consequent = consequent.substitute(&matching);
                    self.queue.push_back(Activation::from_fact(new_consequent));
                }
            }
        }
        self
    }
    fn preprocess_matched_rule(&self, matching: SynMatching,
                               antecedents: Vec<Fact>,
                               more_antecedents: VecDeque<Vec<Fact>>,
                               consequents: Vec<Fact>) -> Rule {
        let new_antecedents = antecedents.iter()
                                         .map(|antecedent| antecedent.substitute(&matching))
                                         .collect();
        let mut new_more_antecedents = VecDeque::new();
        for more_ants in more_antecedents {
            new_more_antecedents.push_back(more_ants.iter()
                                                    .map(|antecedent| antecedent.substitute(&matching))
                                                    .collect())
        }
        let new_consequents = consequents.iter()
                                         .map(|consequent| consequent.substitute(&matching))
                                         .collect();
        Rule {
            antecedents: new_antecedents,
            more_antecedents: new_more_antecedents,
            consequents: new_consequents
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kb_1() {
        let mut kb = KnowledgeBase::new();
        kb = kb.tell("susan ISA person.");
        let resp = kb.ask("susan ISA person.");
        assert!(resp);
    }
    #[test]
    fn kb_1_1() {
        let mut kb = KnowledgeBase::new();
        kb = kb.tell("susan ISA (what: person, kind: female).");
        let resp = kb.ask("susan ISA (what: person, kind: female).");
        assert!(resp);
    }
    #[test]
    fn kb_1_2() {
        let mut kb = KnowledgeBase::new();
        kb = kb.tell("susan ISA (what: person, kind: female).");
        let resp = kb.ask("susan ISA (what: person, kind: <X0>).");
        assert!(resp);
    }
    #[test]
    fn kb_2() {
        let mut kb = KnowledgeBase::new();
        kb = kb.tell("susan ISA person.");
        let mut resp = kb.ask("susan ISA person.");
        assert!(resp);
        resp = kb.ask("susan ISA walrus.");
        assert!(!resp);
    }
    #[test]
    fn kb_3() {
        let mut kb = KnowledgeBase::new();
        kb = kb.tell("susan ISA person.");
        kb = kb.tell("susan ISA animal.");
        let mut resp = kb.ask("susan ISA person.");
        assert!(resp);
        resp = kb.ask("susan ISA animal.");
        assert!(resp);
        resp = kb.ask("susan ISA walrus.");
        assert!(!resp);
    }
    #[test]
    fn kb_3_1() {
        let mut kb = KnowledgeBase::new();
        kb = kb.tell("susan ISA person.");
        kb = kb.tell("peter ISA animal.");
        let mut resp = kb.ask("susan ISA person.");
        assert!(resp);
        resp = kb.ask("peter ISA animal.");
        assert!(resp);
        resp = kb.ask("susan ISA walrus.");
        assert!(!resp);
    }
    #[test]
    fn kb_3_2() {
        let mut kb = KnowledgeBase::new();
        kb = kb.tell("susan ISA person.");
        kb = kb.tell("susan IS animal.");
        let mut resp = kb.ask("susan ISA person.");
        assert!(resp);
        resp = kb.ask("susan IS animal.");
        assert!(resp);
        resp = kb.ask("susan ISA walrus.");
        assert!(!resp);
    }
    #[test]
    fn kb_4_0() {
        let mut kb = KnowledgeBase::new();
        kb = kb.tell("<X0> ISA <X1>; <X1> IS <X2> -> <X0> ISA <X2>.");
        kb = kb.tell("susan ISA person.");
        kb = kb.tell("person IS animal.");
        let resp = kb.ask("susan ISA animal.");
        assert!(resp);
    }
    #[test]
    fn kb_4() {
        let mut kb = KnowledgeBase::new();
        kb = kb.tell("<X0> ISA <X1>; <X1> IS <X2> -> <X0> ISA <X2>.");
        kb = kb.tell("<X0> IS <X1>; <X1> IS <X2> -> <X0> IS <X2>.");
        kb = kb.tell("animal IS thing.");
        kb = kb.tell("mammal IS animal.");
        kb = kb.tell("carnivore IS mammal.");
        kb = kb.tell("human IS carnivore.");
        kb = kb.tell("susan ISA human.");
        let mut resp = kb.ask("susan ISA human.");
        assert!(resp);
        resp = kb.ask("susan ISA animal.");
        assert!(resp);
        resp = kb.ask("susan ISA thing.");
        assert!(resp);
    }
    #[test]
    fn kb_4_1() {
        let mut kb = KnowledgeBase::new();
        kb = kb.tell("<X0> ISA carnivore;\
                      <X1> ISA lamb;
                      (located: <X0>, near: <X1>) ISA fact
                        -> \
                      (eat: <X0>, what: <X1>) ISA fact.");
        kb = kb.tell("lobo ISA carnivore.");
        kb = kb.tell("melinda ISA lamb.");
        kb = kb.tell("(located: lobo, near: melinda) ISA fact.");
        let resp = kb.ask("(eat: lobo, what: melinda) ISA fact.");
        assert!(resp);
    }
    #[test]
    fn kb_5_0() {
        let mut kb = KnowledgeBase::new();
        kb = kb.tell("<X4> ISA (hom: <X2>, hom: <X2>)\
                       -> \
                      <X2> ISA <X4>.");
    }
    #[test]
    fn kb_5() {
        let mut kb = KnowledgeBase::new();
        kb = kb.tell("<X4> ISA (hom1: <X2>, hom2: <X2>);\
                      <X5> ISA (hom1: <X3>, hom2: <X3>);\
                      (p1: <X4>, p2: <X5>) ISA (hom1: (p1: <X2>, p2: <X3>), hom2: (p1: <X2>, p2: <X3>));\
                      (p1: <X6>, p2: <X8>) ISA (fn: <X1>, on: (p1: <X2>, p2: <X3>));\
                      (fn: (fn: <X1>, on: <X4>), on: <X6>) EQ <X7>;\
                      (fn: (fn: <X1>, on: <X5>), on: <X8>) EQ <X9>\
                       -> \
                      (p1: <X7>, p2: <X9>) ISA (fn: <X1>, on: (p1: <X2>, p2: <X3>));\
                      (fn: (fn: <X1>, on: (p1: <X4>, p2: <X5>)), on: (p1: <X6>, p2: <X8>)) EQ (p1: <X7>, p2: <X9>).");
        kb = kb.tell("(p1: <X2>, p2: <X3>) ISA (fn: <X1>, on: (p1: <X4>, p2: <X5>))\
                     -> \
                     <X2> ISA (fn: <X1>, on: <X4>);\
                     <X3> ISA (fn: <X1>, on: <X5>).");
        kb = kb.tell("<X1> ISA (fn: pr, on: nat)\
                     -> \
                     (fn: (fn: pr, on: s1), on: <X1>) EQ (s: <X1>).");
        kb = kb.tell("s2 ISA (hom1: people, hom2: people).");
        kb = kb.tell("(p1: s1, p2: s2) ISA (hom1: (p1: nat, p2: people), hom2: (p1: nat, p2: people)).");
        kb = kb.tell("s1 ISA (hom1: nat, hom2: nat).");
        kb = kb.tell("john ISA (fn: pr, on: people).\
                      susan ISA (fn: pr, on: people).\
                      peter ISA (fn: pr, on: people).");
        kb = kb.tell("(fn: (fn: pr, on: s2), on: john) EQ susan.\
                     (fn: (fn: pr, on: s2), on: susan) EQ peter.");
        kb = kb.tell("(p1: (s: 0), p2: john) ISA (fn: pr, on: (p1: nat, p2: people)).");
        let mut resp = kb.ask("s1 ISA (hom1: nat, hom2: nat).");
        assert!(resp);
        resp = kb.ask("(s: (s: (s: 0))) ISA (fn: pr, on: nat).");
        assert!(resp);
        resp = kb.ask("(fn: (fn: pr, on: s1), on: (s: (s: (s: 0)))) EQ (s: (s: (s: (s: 0)))).");
        assert!(resp);
        resp = kb.ask("(p1: (s: (s: 0)), p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
        assert!(resp);
        let resp2 = kb.ask("(p1: (s: <X1>), p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
        assert!(resp2);
        let resp3 = kb.ask("(p1: <X1>, p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
        assert!(resp3);
    }
}

        // # self.kb.tell("people ISA object.")
        // self.kb.tell("s2 ISA h(hom people,people).")

        // # self.kb.tell("p(nat X people) ISA object.")
        // self.kb.tell("p(s1 X s2) ISA h(hom p(nat X people),p(nat X people)).")

        // # self.kb.tell("pr ISA presheaf.")

        // # self.kb.tell("nat ISA object.")

        // self.kb.tell("s1 ISA h(hom nat,nat).")

        // self.kb.tell("""
        //               john ISA (pr people).
        //               susan ISA (pr people).
        //               mary ISA (pr people).
        //               peter ISA (pr people).
        //               """)

        // self.kb.tell("""
        //              ((pr s2) john) EQ susan.
        //              ((pr s2) susan) EQ mary.
        //              ((pr s2) mary) EQ peter.
        //               """)

        // self.kb.tell("""
        //              p([s,0] X john) ISA (pr p(nat X people))
        //               """)

        // /*  */resp = self.kb.query("p(<X1> X susan) ISA (pr p(nat X people))")