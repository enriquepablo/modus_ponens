use std::collections::HashMap;

use pest::error::Error;
use crate::pest::Parser;
use crate::pest::iterators::Pair;

use crate::constants;
use crate::segment::SynSegment;
use crate::lexicon::Lexicon;
use crate::path::SynPath;
use crate::fact::{ Fact, FLexicon };
use crate::ruletree::Rule as SynRule;
use crate::matching::{ SynMatching };


pub struct ParseResult<'a> {
    pub facts: Vec<&'a Fact<'a>>,
    pub rules: Vec<SynRule<'a>>,
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct Grammar<'a> {
    pub lexicon: Box<Lexicon>,
    pub flexicon: Box<FLexicon<'a>>,
}

impl<'a> Grammar<'a> {

    pub fn new() -> Grammar<'a> {
        Grammar {
            lexicon: Box::new(Lexicon::new()),
            flexicon: Box::new(FLexicon::new()),
        }
    }

    pub fn parse_text(&'a self, text: &'a str) -> Result<ParseResult<'a>, Error<Rule>> {
        let parse_tree = Grammar::parse(Rule::knowledge, text)?.next().unwrap();
        let mut facts: Vec<&'a Fact> = vec![];
        let mut rules: Vec<SynRule> = vec![];
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
                    let rule = SynRule {
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
        let parse_tree = Grammar::parse(Rule::fact, text).ok().unwrap().next().unwrap();
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
                        root_segments: Vec<&'a SynSegment>,
                        mut all_paths: Box<Vec<SynPath<'a>>>,
                        index: usize,
                    ) -> Box<Vec<SynPath>> {
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
            let new_path = SynPath::new(segments);
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
    pub fn substitute_fact(&'a self, fact: &'a Fact<'a>, matching: &SynMatching<'a>) -> &'a Fact<'a> {
        let new_paths = SynPath::substitute_paths(&fact.paths, matching);
        let text = new_paths.iter()
                            .map(|path| path.value.text.as_str())
                            .collect::<Vec<&str>>()
                            .concat();

        // XXX LEAK!
        let stext = Box::leak(text.into_boxed_str());
        
        let parse_tree = Grammar::parse(Rule::fact, stext).ok().unwrap().next().unwrap();
        let all_paths = Box::new(Vec::with_capacity(fact.paths.len()));
        let all_paths = self.visit_parse_node(parse_tree,
                                                                 vec![],
                                                                 all_paths,
                                                                 0);
        self.flexicon.from_paths_and_string(stext, *all_paths)
    }
    pub fn substitute_fact_fast(&'a self, fact: &'a Fact, matching: SynMatching<'a>) -> &'a Fact<'a> {
        let new_paths = SynPath::substitute_paths_owning(&fact.paths, matching);
        self.flexicon.from_paths(new_paths)
    }
    pub fn normalize_fact (&'a self, fact: &'a Fact<'a>) -> (SynMatching<'a>, &'a Fact<'a>) {
        let mut varmap: SynMatching<'a> = HashMap::new();
        let mut invarmap: SynMatching<'a> = HashMap::new();
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fact_1() {
        let grammar = Grammar::new();
        let f1 = grammar.parse_text("susan ISA person.");
        let facts = f1.ok().unwrap().facts;
        let fact = facts.first().unwrap();
        let first = fact.paths.get(0);
        assert!(fact.text == "susan ISA person");
        assert_eq!(format!("{:?}", first.unwrap().value.text), "\"susan\"");

    }

    #[test]
    fn rule_1() {
        let grammar = Grammar::new();
        let f1 = grammar.parse_text("susan ISA person -> susan ISA monkey.");
        let rules = f1.ok().unwrap().rules;
        let rule = rules.first().unwrap();
        let SynRule {
            antecedents,
            consequents, ..
        } = rule;
        {
            let fact = antecedents.get(0).unwrap();
            let first = fact.paths.get(0);
            assert!(fact.text == "susan ISA person");
            assert_eq!(format!("{:?}", first.unwrap().value.text), "\"susan\"");
        }
        {
            let fact = consequents.get(0).unwrap();
            let first = fact.paths.get(0);
            assert!(fact.text == "susan ISA monkey");
            assert_eq!(format!("{:?}", first.unwrap().value.text), "\"susan\"");
        }
    }
}

