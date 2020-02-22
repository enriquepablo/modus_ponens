use pest::error::Error;
use crate::pest::Parser;
use crate::pest::iterators::Pair;

use crate::constants;
use crate::segment::SynSegment;
use crate::path::SynPath;
use crate::fact::Fact;
use crate::ruletree::Rule as SynRule;


#[derive(Parser)]
#[grammar = "grammar.pest"]
struct SynParser;


#[derive(Debug)]
struct FactBuilder<'a> {
    parse_tree: Pair<'a, Rule>,
    root_segments: &'a Vec<SynSegment>,
    all_paths: Box<Vec<SynPath>>,
    index: usize,
}

impl<'a> FactBuilder<'a> {

    fn visit_parse_node(self) -> Box<Vec<SynPath>> {
        let FactBuilder {
            parse_tree, root_segments,
            mut all_paths, index,
        } = self;
        let rule = parse_tree.as_rule();
        let name = format!("{:?}", rule);
        let mut text = String::from(parse_tree.as_str());
        let can_be_var = name.starts_with(constants::VAR_RANGE_PREFIX);
        let children: Vec<_> = parse_tree.into_inner().collect();
        let is_leaf = children.len() == 0;
        if !can_be_var && !is_leaf {
            text = format!("{:?}", index);
        }
        let segment = SynSegment::new(&name, &text, is_leaf);
        let mut new_root_segments = root_segments.clone();
        new_root_segments.push(segment);
        if can_be_var || is_leaf {
            let segments = new_root_segments.clone();
            let new_path = SynPath::new(segments);
            all_paths.push(new_path);
        }
        let mut new_index = 0;
        for child in children {
            let builder = FactBuilder {
                parse_tree: child,
                root_segments: &new_root_segments,
                all_paths, index: new_index,
            };
            new_index += 1;
            all_paths = builder.visit_parse_node();
        }
        all_paths
    }
}

pub fn build_fact(parse_tree: Pair<Rule>) -> Fact {
    let root_segments = vec![];
    let fact_str = String::from(parse_tree.as_str());
    let mut all_paths = Box::new(vec![]);
    let builder = FactBuilder {
        parse_tree,
        root_segments: &root_segments,
        all_paths,
        index: 0,
    };
    all_paths = builder.visit_parse_node();
    Fact {
        text: fact_str,
        paths: *all_paths,
    }
}


pub struct ParseResult {
    pub facts: Vec<Fact>,
    pub rules: Vec<SynRule>,
}


pub fn parse_text(text: &str) -> Result<ParseResult, Error<Rule>> {
    let parse_tree = SynParser::parse(Rule::knowledge, text)?.next().unwrap();
    let mut facts: Vec<Fact> = vec![];
    let mut rules: Vec<SynRule> = vec![];
    for pair in parse_tree.into_inner() {
        match pair.as_rule() {
            Rule::fact => {
                let fact = build_fact(pair);
                facts.push(fact);
            },
            Rule::rule => {
                let mut antecedents = vec![];
                let mut consequents = vec![];
                for pairset in pair.into_inner() {
                    match pairset.as_rule() {
                        Rule::antecedents => {
                            for factpair in pairset.into_inner() {
                                let antecedent = build_fact(factpair);
                                antecedents.push(antecedent);
                            }
                        },
                        Rule::consequents => {
                            for factpair in pairset.into_inner() {
                                let consequent = build_fact(factpair);
                                consequents.push(consequent);
                            }
                        },
                        _ => {}
                    }
                }
                let rule = SynRule { antecedents, consequents};
                rules.push(rule);
            },
            _ => {}
        }
    }
    Ok(ParseResult { facts, rules })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fact_1() {
        let f1 = parse_text("susan ISA person.");
        let facts = f1.ok().unwrap().facts;
        let fact = facts.first().unwrap();
        let Fact {
            text, paths
        } = fact;
        let first = paths.get(0);
        assert!(text == "susan ISA person");
        assert_eq!(format!("{:?}", first.unwrap().value.text), "\"susan\"");

    }

    #[test]
    fn rule_1() {
        let f1 = parse_text("susan ISA person -> susan ISA monkey.");
        let rules = f1.ok().unwrap().rules;
        let rule = rules.first().unwrap();
        let SynRule {
            antecedents, consequents
        } = rule;
        {
            let fact = antecedents.get(0).unwrap();
            let Fact {
                text, paths
            } = fact;
            let first = paths.get(0);
            assert!(text == "susan ISA person");
            assert_eq!(format!("{:?}", first.unwrap().value.text), "\"susan\"");
        }
        {
            let fact = consequents.get(0).unwrap();
            let Fact {
                text, paths
            } = fact;
            let first = paths.get(0);
            assert!(text == "susan ISA monkey");
            assert_eq!(format!("{:?}", first.unwrap().value.text), "\"susan\"");
        }
    }
}

