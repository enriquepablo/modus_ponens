
use crate::ruletree::MPRule;
use crate::fact::Fact;
use crate::matching::MPMatching;

pub struct ParseResult<'a> {
    pub facts: Vec<&'a Fact<'a>>,
    pub rules: Vec<MPRule<'a>>,
}


#[derive(Debug)]
pub enum Activation<'a> {
    MPRule {
        rule: MPRule<'a>,
        query_rules: bool,
    },
    Fact {
        fact: &'a Fact<'a>,
        query_rules: bool,
    },
    Match {
        rule: MPRule<'a>,
        matched: MPMatching<'a>,
        query_rules: bool,
    },
}

impl<'a> Activation<'a> {

    pub fn from_fact(fact: &'a Fact, query_rules: bool) -> Activation<'a> {
        Activation::Fact {
            fact: fact,
            query_rules,
        }
    }
    pub fn from_rule(rule: MPRule, query_rules: bool) -> Activation {
        Activation::MPRule {
            rule: rule,
            query_rules,
        }
    }
    pub fn from_matching(rule: MPRule<'a>, matched: MPMatching<'a>, query_rules: bool) -> Activation<'a> {
        Activation::Match {
            rule: rule,
            matched: matched,
            query_rules,
        }
    }
}
