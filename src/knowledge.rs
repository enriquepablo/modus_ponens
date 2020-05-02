use std::collections::{ VecDeque, HashMap };
use std::cell::RefCell;

use crate::matching::{ SynMatching, get_real_matching_owning };
use crate::fact::Fact;
use crate::factset::FactSet;
use crate::ruletree::{ Rule, RSNode, RuleRef, new_response };
use crate::parser::{ Grammar, ParseResult };


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ActType {
    Rule,
    Fact,
    Match,
}

#[derive(Debug)]
pub struct Activation<'a> {
    atype: ActType,
    rule: Option<Rule<'a>>,
    fact: Option<&'a Fact<'a>>,
    matched: Option<SynMatching<'a>>,
    query_rules: bool,
}

impl<'a> Activation<'a> {

    pub fn from_fact(fact: &'a Fact, query_rules: bool) -> Activation<'a> {
        Activation {
            atype: ActType::Fact,
            rule: None,
            fact: Some(fact),
            matched: None,
            query_rules,
        }
    }
    pub fn from_rule(rule: Rule, query_rules: bool) -> Activation {
        Activation {
            atype: ActType::Rule,
            rule: Some(rule),
            fact: None,
            matched: None,
            query_rules,
        }
    }
    pub fn from_matching(rule: Rule<'a>, matched: SynMatching<'a>, query_rules: bool) -> Activation<'a> {
        Activation {
            atype: ActType::Match,
            rule: Some(rule),
            fact: None,
            matched: Some(matched),
            query_rules,
        }
    }
}

pub struct KStat {
    pub rules: usize,
    pub rules_known: usize,
    pub facts: usize,
    pub facts_known: usize,
}

impl KStat {

    pub fn new () -> KStat {
        KStat {rules: 0, rules_known: 0, facts: 0, facts_known: 0}
    }
}

pub struct KDB<'a> {
    facts: FactSet<'a>,
    rules: RSNode<'a>,
    queue: RefCell<VecDeque<Activation<'a>>>,
}

impl<'a> KDB<'a> {

    pub fn new (grammar: &'a Grammar<'a>) -> KDB<'a> {
        let root_path = grammar.lexicon.empty_path();
        KDB {
            facts: FactSet::new(),
            rules: RSNode::new(root_path),
            queue: RefCell::new(VecDeque::new()),
        }
    }
}

pub struct KnowledgeBase<'a> {
    grammar: &'a Grammar<'a>,
    facts: FactSet<'a>,
    rules: RSNode<'a>,
    queue: RefCell<VecDeque<Activation<'a>>>,
}

impl<'a> KnowledgeBase<'a> {

    pub fn new (grammar: &'a Grammar<'a>) -> KnowledgeBase<'a> {
        let root_path = grammar.lexicon.empty_path();
        KnowledgeBase {
            grammar,
            facts: FactSet::new(),
            rules: RSNode::new(root_path),
            queue: RefCell::new(VecDeque::new()),
        }
    }

    pub fn tell(&'a self, knowledge: &'a str) {
        let result = self.grammar.parse_text(knowledge);
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
            let act = Activation::from_fact(fact, false);
            self.queue.borrow_mut().push_back(act);
            self.process_activations();
        }
    }
    pub fn ask(&'a self, knowledge: &'a str) -> bool {
        let ParseResult { mut facts, .. } = self.grammar.parse_text(knowledge).ok().unwrap();
        let fact = facts.pop().unwrap();
        let resps = self.facts.ask_fact(&fact);
        resps.len() > 0
    }
    fn process_activations(&'a self) {
        while !self.queue.borrow().is_empty() {
            let next = self.queue.borrow_mut().pop_front().unwrap();
            match next {
                Activation {
                    atype: ActType::Fact,
                    fact: Some(fact),
                    query_rules, ..
                } => {
                    self.process_fact(fact, query_rules);
                },
                Activation {
                    atype: ActType::Rule,
                    rule: Some(rule), ..
                } => {
                    self.process_rule(rule);
                },
                Activation {
                    atype: ActType::Match,
                    rule: Some(rule),
                    matched: Some(matched),
                    query_rules, ..
                } => {
                    self.process_match(rule, &matched, query_rules);
                },
                _ => {}
            }
        }
    }
    fn process_rule(&'a self, mut rule: Rule<'a>) {
        
        //println!("ADDING RULE {}", rule);
        let n_ants = rule.antecedents.len();
        for n in 0..n_ants {
            let mut new_ants = vec![];
            let mut new_ant: Option<&Fact> = None;
            let Rule {
                antecedents,
                more_antecedents,
                consequents
            } = rule;
            for (i, ant) in antecedents.iter().enumerate() {
                if n == i {
                    new_ant = Some(*ant);
                } else {
                    new_ants.push(*ant);
                }
            }
            let new_conseqs = consequents.clone();
            let new_more_ants = more_antecedents.iter().cloned().collect();
            let new_rule = Rule {
                antecedents: new_ants,
                more_antecedents: new_more_ants,
                consequents: new_conseqs
            };
            let (varmap, normal_ant) = self.grammar.normalize_fact(&new_ant.unwrap());
            let rule_ref = RuleRef {
                rule: new_rule,
                varmap,
            };
            let normal_leaf_paths = normal_ant.paths.as_slice();
            self.rules.follow_and_create_paths(normal_leaf_paths, rule_ref);
            rule = Rule {
                antecedents,
                more_antecedents,
                consequents
            };
        }
    }
    fn process_fact(&'a self,
                    fact: &'a Fact<'a>,
                    query_rules: bool) {
        
        //println!("ADDING FACT: {}", fact);
        let response = new_response();
        let matched: SynMatching = HashMap::new();
        let paths = fact.paths.as_slice();
        let (response, _) = self.rules.climb(paths, response, matched);
        for (rule_refs, matching) in *response {
            for RuleRef { rule, varmap } in rule_refs {
                let real_matching = get_real_matching_owning(matching.clone(), varmap); 
                self.queue.borrow_mut().push_back(Activation::from_matching(rule, real_matching, query_rules));
            }
        }
        self.facts.add_fact(&fact);
    }
    fn process_match(&'a self,
                     mut rule: Rule<'a>,
                     matching: &SynMatching<'a>,
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
               let new_consequent = self.grammar.substitute_fact(&consequent, matching);
               self.queue.borrow_mut().push_back(Activation::from_fact(new_consequent, query_rules));
           }
        }
    }
    fn query_rule(&'a self,
                  rule: Rule<'a>,
                  query_rules: bool) -> Rule {

        for i in 0..rule.antecedents.len() {
            let mut new_ants = rule.antecedents.clone();
            let ant = new_ants.remove(i);
            let resps = self.facts.ask_fact(ant);
            for resp in resps {
                let new_rule = Rule {
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
                               matching: &SynMatching<'a>,
                               rule: Rule<'a>) -> (Rule<'a>, bool) {
        let Rule {
            mut antecedents,
            mut more_antecedents,
            consequents
        } = rule;
        if antecedents.len() == 0 {
            if more_antecedents.len() == 0 {
                return (Rule {antecedents, more_antecedents, consequents}, false);
            } else {
                antecedents = more_antecedents.remove(0);
            }
        }
        let new_antecedents = antecedents.iter()
                                         .map(|antecedent| self.grammar.substitute_fact(antecedent, matching))
                                         .collect();
        let mut new_more_antecedents = Vec::new();
        while more_antecedents.len() > 0 {
            let more_ants = more_antecedents.remove(0); 
            new_more_antecedents.push(more_ants.iter()
                                               .map(|antecedent| self.grammar.substitute_fact(antecedent, matching))
                                               .collect());
        }
        let new_consequents = consequents.iter()
                                         .map(|consequent| self.grammar.substitute_fact(consequent, matching))
                                         .collect();
        (Rule {
            antecedents: new_antecedents,
            more_antecedents: new_more_antecedents,
            consequents: new_consequents
        }, true)

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kb_1() {
        let grammar = Grammar::new();
        let kb = KnowledgeBase::new(&grammar);
        kb.tell("susan ISA person.");
        let resp = kb.ask("susan ISA person.");
        assert!(resp);
    }
    #[test]
    fn test_kb_1_1() {
        let grammar = Grammar::new();
        let kb = KnowledgeBase::new(&grammar);
        kb.tell("susan ISA (what: person, kind: female).");
        let resp = kb.ask("susan ISA (what: person, kind: female).");
        assert!(resp);
    }
//    #[test]
//    fn kb_1_2() {
//        let grammar = Grammar::new();
//        let mut queue = VecDeque::new();
//        let mut stats = KStat::new();
//        let kdb = KDB::new(&grammar);
//        let mut kb = KnowledgeBase::new(&grammar, &mut queue, &mut stats);
//        kdb = kb.tell(kdb, "susan ISA (what: person, kind: female).");
//        let resp = kb.ask(&kdb, "susan ISA (what: person, kind: <X0>).");
//        assert!(resp);
//    }
//    #[test]
//    fn kb_2() {
//        let grammar = Grammar::new();
//        let mut queue = VecDeque::new();
//        let mut stats = KStat::new();
//        let kdb = KDB::new(&grammar);
//        let mut kb = KnowledgeBase::new(&grammar, &mut queue, &mut stats);
//        kdb = kb.tell(kdb, "susan ISA person.");
//        let mut resp = kb.ask(&kdb, "susan ISA person.");
//        assert!(resp);
//        resp = kb.ask(&kdb, "susan ISA walrus.");
//        assert!(!resp);
//    }
    #[test]
    fn test_kb_3() {
        let grammar = Grammar::new();
        let kb = KnowledgeBase::new(&grammar);
        kb.tell("susan ISA person.");
        kb.tell("susan ISA animal.");
        let resp = kb.ask("susan ISA person.");
        assert!(resp);
        let resp = kb.ask("susan ISA animal.");
        assert!(resp);
        let resp = kb.ask("susan ISA walrus.");
        assert!(!resp);
    }
//    #[test]
//    fn kb_3_1() {
//        let grammar = Grammar::new();
//        let mut queue = VecDeque::new();
//        let mut stats = KStat::new();
//        let kdb = KDB::new(&grammar);
//        let mut kb = KnowledgeBase::new(&grammar, &mut queue, &mut stats);
//        kdb = kb.tell(kdb, "susan ISA person.");
//        kdb = kb.tell(kdb, "peter ISA animal.");
//        let mut resp = kb.ask(&kdb, "susan ISA person.");
//        assert!(resp);
//        resp = kb.ask(&kdb, "peter ISA animal.");
//        assert!(resp);
//        resp = kb.ask(&kdb, "susan ISA walrus.");
//        assert!(!resp);
//    }
//    #[test]
//    fn kb_3_2() {
//        let grammar = Grammar::new();
//        let mut queue = VecDeque::new();
//        let mut stats = KStat::new();
//        let kdb = KDB::new(&grammar);
//        let mut kb = KnowledgeBase::new(&grammar, &mut queue, &mut stats);
//        kdb = kb.tell(kdb, "susan ISA person.");
//        kdb = kb.tell(kdb, "susan IS animal.");
//        let mut resp = kb.ask(&kdb, "susan ISA person.");
//        assert!(resp);
//        resp = kb.ask(&kdb, "susan IS animal.");
//        assert!(resp);
//        resp = kb.ask(&kdb, "susan ISA walrus.");
//        assert!(!resp);
//    }
    #[test]
    fn test_kb_4_0() {
        let grammar = Grammar::new();
        let kb = KnowledgeBase::new(&grammar);
        kb.tell("<X0> ISA <X1>; <X1> IS <X2> -> <X0> ISA <X2>.");
        kb.tell("susan ISA person.");
        kb.tell("person IS animal.");
        let resp = kb.ask( "susan ISA animal.");
        assert!(resp);
    }
//    #[test]
//    fn kb_4() {
//        let grammar = Grammar::new();
//        let mut queue = VecDeque::new();
//        let mut stats = KStat::new();
//        let kdb = KDB::new(&grammar);
//        let mut kb = KnowledgeBase::new(&grammar, &mut queue, &mut stats);
//        kdb = kb.tell(kdb, "<X0> ISA <X1>; <X1> IS <X2> -> <X0> ISA <X2>.");
//        kdb = kb.tell(kdb, "<X0> IS <X1>; <X1> IS <X2> -> <X0> IS <X2>.");
//        kdb = kb.tell(kdb, "animal IS thing.");
//        kdb = kb.tell(kdb, "mammal IS animal.");
//        kdb = kb.tell(kdb, "carnivore IS mammal.");
//        kdb = kb.tell(kdb, "human IS carnivore.");
//        kdb = kb.tell(kdb, "susan ISA human.");
//        let mut resp = kb.ask(&kdb, "susan ISA human.");
//        assert!(resp);
//        resp = kb.ask(&kdb, "susan ISA animal.");
//        assert!(resp);
//        resp = kb.ask(&kdb, "susan ISA thing.");
//        assert!(resp);
//    }
//    #[test]
//    fn kb_4_1() {
//        let grammar = Grammar::new();
//        let mut queue = VecDeque::new();
//        let mut stats = KStat::new();
//        let kdb = KDB::new(&grammar);
//        let mut kb = KnowledgeBase::new(&grammar, &mut queue, &mut stats);
//        kdb = kb.tell(kdb, "<X0> ISA carnivore;\
//                      <X1> ISA lamb;
//                      (located: <X0>, near: <X1>) ISA fact
//                        -> \
//                      (eat: <X0>, what: <X1>) ISA fact.");
//        kdb = kb.tell(kdb, "lobo ISA carnivore.");
//        kdb = kb.tell(kdb, "melinda ISA lamb.");
//        kdb = kb.tell(kdb, "(located: lobo, near: melinda) ISA fact.");
//        let resp = kb.ask(&kdb, "(eat: lobo, what: melinda) ISA fact.");
//        assert!(resp);
//    }
//    #[test]
//    fn kb_5_0() {
//        let grammar = Grammar::new();
//        let mut queue = VecDeque::new();
//        let mut stats = KStat::new();
//        let kdb = KDB::new(&grammar);
//        let kb = KnowledgeBase::new(&grammar, &mut queue, &mut stats);
//        kb.tell(kdb, "<X4> ISA (hom: <X2>, hom: <X2>)\
//                  -> \
//                 <X2> ISA <X4>.");
//    }
//    #[test]
//    #[ignore]
//    fn kb_5() {
//        let grammar = Grammar::new();
//        let mut queue = VecDeque::new();
//        let mut stats = KStat::new();
//        let kdb = KDB::new(&grammar);
//        let mut kb = KnowledgeBase::new(&grammar, &mut queue, &mut stats);
//        kdb = kb.tell(kdb, "<X4> ISA (hom1: <X2>, hom2: <X2>);\
//                      <X5> ISA (hom1: <X3>, hom2: <X3>);\
//                      (p1: <X4>, p2: <X5>) ISA (hom1: (p1: <X2>, p2: <X3>), hom2: (p1: <X2>, p2: <X3>));\
//                      (p1: <X6>, p2: <X8>) ISA (fn: <X1>, on: (p1: <X2>, p2: <X3>));\
//                      (fn: (fn: <X1>, on: <X4>), on: <X6>) EQ <X7>;\
//                      (fn: (fn: <X1>, on: <X5>), on: <X8>) EQ <X9>\
//                       -> \
//                      (p1: <X7>, p2: <X9>) ISA (fn: <X1>, on: (p1: <X2>, p2: <X3>));\
//                      (fn: (fn: <X1>, on: (p1: <X4>, p2: <X5>)), on: (p1: <X6>, p2: <X8>)) EQ (p1: <X7>, p2: <X9>).");
//        kdb = kb.tell(kdb, "(p1: <X2>, p2: <X3>) ISA (fn: <X1>, on: (p1: <X4>, p2: <X5>))\
//                     -> \
//                     <X2> ISA (fn: <X1>, on: <X4>);\
//                     <X3> ISA (fn: <X1>, on: <X5>).");
//        kdb = kb.tell(kdb, "<X1> ISA (fn: pr, on: nat)\
//                     -> \
//                     (fn: (fn: pr, on: s1), on: <X1>) EQ (s: <X1>).");
//        kdb = kb.tell(kdb, "s2 ISA (hom1: people, hom2: people).");
//        kdb = kb.tell(kdb, "(p1: s1, p2: s2) ISA (hom1: (p1: nat, p2: people), hom2: (p1: nat, p2: people)).");
//        kdb = kb.tell(kdb, "s1 ISA (hom1: nat, hom2: nat).");
//        kdb = kb.tell(kdb, "john ISA (fn: pr, on: people).\
//                      susan ISA (fn: pr, on: people).\
//                      peter ISA (fn: pr, on: people).");
//        kdb = kb.tell(kdb, "(fn: (fn: pr, on: s2), on: john) EQ susan.\
//                     (fn: (fn: pr, on: s2), on: susan) EQ peter.");
//        kdb = kb.tell(kdb, "(p1: (s: 0), p2: john) ISA (fn: pr, on: (p1: nat, p2: people)).");
//        let mut resp = kb.ask(&kdb, "s1 ISA (hom1: nat, hom2: nat).");
//        assert!(resp);
//        resp = kb.ask(&kdb, "(s: (s: (s: 0))) ISA (fn: pr, on: nat).");
//        assert!(resp);
//        resp = kb.ask(&kdb, "(fn: (fn: pr, on: s1), on: (s: (s: (s: 0)))) EQ (s: (s: (s: (s: 0)))).");
//        assert!(resp);
//        resp = kb.ask(&kdb, "(p1: (s: (s: 0)), p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
//        assert!(resp);
//        let resp2 = kb.ask(&kdb, "(p1: (s: <X1>), p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
//        assert!(resp2);
//        let resp3 = kb.ask(&kdb, "(p1: <X1>, p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
//        assert!(resp3);
//    }
    #[test]
    fn kb_6() {
        let grammar = Grammar::new();
        let kb = KnowledgeBase::new(&grammar);
        kb.tell("(p1: <X4>, p2: <X5>) ISA (hom1: (p1: <X2>, p2: <X3>), hom2: (p1: <X2>, p2: <X3>))\
                      -> \
                      <X4> ISA (hom1: <X2>, hom2: <X2>);\
                      <X5> ISA (hom1: <X3>, hom2: <X3>)\
                      -> \
                      (p1: <X6>, p2: <X8>) ISA (fn: <X1>, on: (p1: <X2>, p2: <X3>))\
                      -> \
                      (fn: (fn: <X1>, on: <X4>), on: <X6>) EQ <X7>;\
                      (fn: (fn: <X1>, on: <X5>), on: <X8>) EQ <X9>\
                       -> \
                      (p1: <X7>, p2: <X9>) ISA (fn: <X1>, on: (p1: <X2>, p2: <X3>));\
                      (fn: (fn: <X1>, on: (p1: <X4>, p2: <X5>)), on: (p1: <X6>, p2: <X8>)) EQ (p1: <X7>, p2: <X9>).");
        kb.tell("(p1: <X2>, p2: <X3>) ISA (fn: <X1>, on: (p1: <X4>, p2: <X5>))\
                     -> \
                     <X2> ISA (fn: <X1>, on: <X4>);\
                     <X3> ISA (fn: <X1>, on: <X5>).");
        kb.tell("<X1> ISA (fn: pr, on: nat)\
                     -> \
                     (fn: (fn: pr, on: s1), on: <X1>) EQ (s: <X1>).");
        kb.tell("s2 ISA (hom1: people, hom2: people).");
        kb.tell("(p1: s1, p2: s2) ISA (hom1: (p1: nat, p2: people), hom2: (p1: nat, p2: people)).");
        kb.tell("s1 ISA (hom1: nat, hom2: nat).");
        kb.tell("(p1: (s: 0), p2: john) ISA (fn: pr, on: (p1: nat, p2: people)).");
        kb.tell("john ISA (fn: pr, on: people).\
                      susan ISA (fn: pr, on: people).\
                      sue1 ISA (fn: pr, on: people).\
                      sue2 ISA (fn: pr, on: people).\
                      sue3 ISA (fn: pr, on: people).\
                      sue4 ISA (fn: pr, on: people).\
                      sue5 ISA (fn: pr, on: people).\
                      sue6 ISA (fn: pr, on: people).\
                      sue7 ISA (fn: pr, on: people).\
                      sue8 ISA (fn: pr, on: people).\
                      sue9 ISA (fn: pr, on: people).\
                      sue10 ISA (fn: pr, on: people).\
                      sue11 ISA (fn: pr, on: people).\
                      sue12 ISA (fn: pr, on: people).\
                      sue13 ISA (fn: pr, on: people).\
                      sue14 ISA (fn: pr, on: people).\
                      sue15 ISA (fn: pr, on: people).\
                      sue16 ISA (fn: pr, on: people).\
                      sue17 ISA (fn: pr, on: people).\
                      sue18 ISA (fn: pr, on: people).\
                      sue19 ISA (fn: pr, on: people).\
                      ken ISA (fn: pr, on: people).\
                      bob ISA (fn: pr, on: people).\
                      isa ISA (fn: pr, on: people).\
                      peter ISA (fn: pr, on: people).");
        kb.tell("(fn: (fn: pr, on: s2), on: john) EQ susan.\
                     (fn: (fn: pr, on: s2), on: susan) EQ sue1.\
                     (fn: (fn: pr, on: s2), on: sue1) EQ sue2.\
                     (fn: (fn: pr, on: s2), on: sue2) EQ sue3.\
                     (fn: (fn: pr, on: s2), on: sue3) EQ sue4.\
                     (fn: (fn: pr, on: s2), on: sue4) EQ sue5.\
                     (fn: (fn: pr, on: s2), on: sue5) EQ sue6.\
                     (fn: (fn: pr, on: s2), on: sue6) EQ sue7.\
                     (fn: (fn: pr, on: s2), on: sue7) EQ sue8.\
                     (fn: (fn: pr, on: s2), on: sue8) EQ sue9.\
                     (fn: (fn: pr, on: s2), on: sue9) EQ sue10.\
                     (fn: (fn: pr, on: s2), on: sue10) EQ sue11.\
                     (fn: (fn: pr, on: s2), on: sue11) EQ sue12.\
                     (fn: (fn: pr, on: s2), on: sue12) EQ sue13.\
                     (fn: (fn: pr, on: s2), on: sue13) EQ sue14.\
                     (fn: (fn: pr, on: s2), on: sue14) EQ sue15.\
                     (fn: (fn: pr, on: s2), on: sue15) EQ sue16.\
                     (fn: (fn: pr, on: s2), on: sue16) EQ sue17.\
                     (fn: (fn: pr, on: s2), on: sue17) EQ sue18.\
                     (fn: (fn: pr, on: s2), on: sue18) EQ sue19.\
                     (fn: (fn: pr, on: s2), on: sue19) EQ ken.\
                     (fn: (fn: pr, on: s2), on: ken) EQ bob.\
                     (fn: (fn: pr, on: s2), on: bob) EQ isa.\
                     (fn: (fn: pr, on: s2), on: isa) EQ peter.");
        let resp = kb.ask("s1 ISA (hom1: nat, hom2: nat).");
        assert!(resp);
        let resp = kb.ask("(s: (s: (s: (s: (s: (s: (s: (s: (s: 0))))))))) ISA (fn: pr, on: nat).");
        assert!(resp);
        let resp = kb.ask("(fn: (fn: pr, on: s1), on: (s: (s: (s: 0)))) EQ (s: (s: (s: (s: 0)))).");
        assert!(resp);
        let resp = kb.ask("(p1: (s: (s: 0)), p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
        assert!(resp);
        let resp2 = kb.ask("(p1: (s: <X1>), p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
        assert!(resp2);
        let resp3 = kb.ask("(p1: <X1>, p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
        assert!(resp3);
    }
}