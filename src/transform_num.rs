use pest::Parser;
//use log::debug;

use crate::lexicon::Lexicon;
use crate::matching::MPMatching;
use crate::segment::MPSegment;

#[derive(Parser)]
#[grammar = "transform_num.pest"]
pub struct TNParser<'a> {
    lexicon: &'a Lexicon,
}

impl<'a> TNParser<'a> {

    pub fn new(lexicon: &'a Lexicon) -> TNParser<'a> {
        TNParser {
            lexicon
        }
    }

    pub fn compile(&self, source: &str, matching: &MPMatching<'a>) -> &MPSegment {
        let parse_result = TNParser::parse(Rule::expr, source);
        if parse_result.is_err() {
            panic!("These do not seem like transforms: \"{}\"\n\nerr: {}\n\nmatching: {:?}", source, parse_result.err().unwrap(), matching);
        }
        let pair = parse_result.ok().unwrap().next().unwrap();
        let num = format!("{}", self.compile_num(pair, matching));
        self.lexicon.intern_with_text("v_decimal", num, true)
    }

    fn compile_num(&self, pair: pest::iterators::Pair<Rule>, matching: &MPMatching<'a>) -> f64 {
        match pair.as_rule() {
            Rule::expr => {
                self.compile_num(pair.into_inner().next().expect("cua"), matching)
            },
            Rule::monadicExpr => {
                let mut pair = pair.into_inner();
                let op = pair.next().expect("cin");
                let termpair = pair.next().expect("sei");
                let term = self.compile_num(termpair, matching);
                parse_monadic_op(op, term)
            },
            Rule::dyadicExpr => {
                let mut pair = pair.into_inner();
                let lhspair = pair.next().expect("sie");
                let lhs = self.compile_num(lhspair, matching);
                let op = pair.next().expect("och");
                let rhspair = pair.next().expect("nue");
                let rhs = self.compile_num(rhspair, matching);
                parse_dyadic_op(op, lhs, rhs)
            },
            Rule::v_decimal => {
                pair.as_str().parse::<f64>().ok().expect("die")
            },
            Rule::var => {
                let var = self.lexicon.intern("var", pair.as_str(), true);
                let number = matching.get(var).expect("number segment");
                let result = number.text.parse::<f64>();
                if result.is_err() {
                    panic!("This do not seem like a number: \"{}\"\n\nerr: {}\n\nmatching: {:?}", number.text, result.err().unwrap(), matching);
                }
                result.ok().unwrap()
            },
            unknown_expr => panic!("Unexpected expression: {:?}", unknown_expr),
        }
    }
}

fn parse_dyadic_op(op: pest::iterators::Pair<Rule>, lhs: f64, rhs: f64) -> f64 {
    match op.as_str() {
        "-" => lhs - rhs,
        "+" => lhs + rhs,
        "**" => lhs.powf(rhs),
        "*" => lhs * rhs,
        "/" => lhs / rhs,
        "%" => (lhs % rhs),
        _ => panic!("Unexpected dyadic operator: {}", op.as_str()),
    }
}

fn parse_monadic_op(op: pest::iterators::Pair<Rule>, term: f64) -> f64 {
    match op.as_str() {
        "-" => - term,
        "log" => term.log2(),
        "exp" => - term.exp(),
        "sin" => - term.sin(),
        "cos" => - term.cos(),
        "tan" => - term.tan(),
        "floor" => - term.floor(),
        "ceil" => - term.ceil(),
        "asin" => - term.asin(),
        "acos" => - term.acos(),
        "atan" => - term.atan(),
        _ => panic!("Unexpected dyadic verb: {}", op.as_str()),
    }
}

