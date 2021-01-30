use pest::Parser;
//use log::debug;

use crate::lexicon::Lexicon;
use crate::matching::MPMatching;
use crate::segment::MPSegment;
use crate::transform_str::TSParser;
use crate::transform_num::TNParser;

#[derive(Parser)]
#[grammar = "transform.pest"]
pub struct TParser<'a> {
    lexicon: &'a Lexicon,
    num_parser: TNParser<'a>,
    str_parser: TSParser<'a>,
}

impl<'a> TParser<'a> {

    pub fn new(lexicon: &'a Lexicon) -> TParser<'a> {
        let num_parser = TNParser::new(lexicon);
        let str_parser = TSParser::new(lexicon);
        TParser {
            lexicon, num_parser, str_parser
        }
    }

    pub fn process_transforms(&'a self, source: &'a str, mut matching: MPMatching<'a>) -> MPMatching<'a> {
        let mut var: &MPSegment;
        let mut val: &MPSegment;

        let parse_result = TParser::parse(Rule::transforms, source);
        if parse_result.is_err() {
            panic!("These do not seem like transforms: \"{}\"\n\nerr: {}\n\nmatching: {:?}", source, parse_result.err().unwrap(), matching);
        }
        let mut pairs = parse_result.ok().unwrap();

        for pair in pairs.next().unwrap().into_inner() {
            val = match pair.as_rule() {
                Rule::num_transform => {
                    let mut asspair = pair.into_inner();
                    let varpair = asspair.next().expect("a variable");
                    var = self.lexicon.intern("var", varpair.as_str(), true);
                    let exprpair = asspair.next().expect("an expression");
                    self.num_parser.compile(exprpair.as_str(), &matching)
                },
                Rule::str_transform => {
                    let mut asspair = pair.into_inner();
                    let varpair = asspair.next().expect("a variable");
                    var = self.lexicon.intern("var", varpair.as_str(), true);
                    let exprpair = asspair.next().expect("an expression");
                    self.str_parser.compile(exprpair.as_str(), &matching)
                },
                unknown_expr => panic!("Unexpected expression: {:?}", unknown_expr),
            };
            matching.insert(var, val);
        }
        matching
    }
}
