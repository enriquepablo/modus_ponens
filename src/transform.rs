use pest::Parser;
//use log::debug;

use crate::lexicon::Lexicon;
use crate::matching::MPMatching;
use crate::segment::MPSegment;

#[derive(Parser)]
#[grammar = "transform.pest"]
pub struct TParser<'a> {
    lexicon: &'a Lexicon,
}

impl<'a> TParser<'a> {

    pub fn new(lexicon: &'a Lexicon) -> TParser<'a> {
        TParser {
            lexicon
        }
    }

    pub fn process_transforms(&self, source: &str, mut matching: MPMatching<'a>) -> MPMatching<'a> {
        let mut var: &MPSegment;
        let mut val: &MPSegment;

        let parse_result = TParser::parse(Rule::transforms, source);
        if parse_result.is_err() {
            panic!("These do not seem like transforms: \"{}\"\n\nerr: {}\n\nmatching: {:?}", source, parse_result.err().unwrap(), matching);
        }
        let mut pairs = parse_result.ok().unwrap();

        for pair in pairs.next().unwrap().into_inner() {
            let mut asspair = pair.into_inner();
            let varpair = asspair.next().expect("dos");
            var = self.lexicon.intern("var", varpair.as_str(), true);
            let exprpair = asspair.next().expect("tre");
            let new_val = self.compile_expr(exprpair, &matching);
            let new_str = format!("{}", new_val);
            val = self.lexicon.intern("v_decimal", new_str.as_str(), true);
            matching.insert(var, val);
        }
        matching
    }

    fn compile_expr(&self, pair: pest::iterators::Pair<Rule>, matching: &MPMatching<'a>) -> f64 {
        match pair.as_rule() {
            Rule::v_expr => {
                self.compile_expr(pair.into_inner().next().expect("cua"), matching)
            },
            Rule::monadicExpr => {
                let mut pair = pair.into_inner();
                let op = pair.next().expect("cin");
                let termpair = pair.next().expect("sei");
                let term = self.compile_expr(termpair, matching);
                parse_monadic_op(op, term)
            },
            Rule::dyadicExpr => {
                let mut pair = pair.into_inner();
                let lhspair = pair.next().expect("sie");
                let lhs = self.compile_expr(lhspair, matching);
                let op = pair.next().expect("och");
                let rhspair = pair.next().expect("nue");
                let rhs = self.compile_expr(rhspair, matching);
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
