use pest::Parser;

use crate::lexicon::Lexicon;
use crate::matching::MPMatching;
use crate::segment::MPSegment;

#[derive(Parser)]
#[grammar = "transform.pest"]
pub struct TParser;

impl<'a> TParser {

    pub fn process_transforms(source: &str, mut matching: MPMatching<'a>, lexicon: &'a Lexicon) -> MPMatching<'a> {
        let mut var: &MPSegment;
        let mut val: &MPSegment;

        let pairs = TParser::parse(Rule::transforms, source).ok().unwrap();

        for pair in pairs {
            let mut asspair = pair.into_inner();
            let varpair = asspair.next().unwrap();
            var = lexicon.intern("var", varpair.as_str(), true);
            asspair.next().unwrap();
            let exprpair = asspair.next().unwrap();
            let (new_val, new_matching) = TParser::compile_expr(exprpair, matching, lexicon);
            matching = new_matching;
            let new_str = new_val.to_string();
            val = lexicon.intern("v_decimal", new_str.as_str(), true);
            matching.insert(var, val);
        }
        matching
    }

    fn compile_expr(pair: pest::iterators::Pair<Rule>, matching: MPMatching<'a>, lexicon: &Lexicon) -> (f64, MPMatching<'a>) {
        match pair.as_rule() {
            Rule::v_expr => {
                TParser::compile_expr(pair.into_inner().next().unwrap(), matching, lexicon)
            },
            Rule::monadicExpr => {
                let mut pair = pair.into_inner();
                let op = pair.next().unwrap();
                let termpair = pair.next().unwrap();
                let (term, new_matching) = TParser::compile_expr(termpair, matching, lexicon);
                (parse_monadic_op(op, term), new_matching)
            },
            Rule::dyadicExpr => {
                let mut pair = pair.into_inner();
                let lhspair = pair.next().unwrap();
                let (lhs, new_matching) = TParser::compile_expr(lhspair, matching, lexicon);
                let op = pair.next().unwrap();
                let rhspair = pair.next().unwrap();
                let (rhs, new_matching) = TParser::compile_expr(rhspair, new_matching, lexicon);
                (parse_dyadic_op(op, lhs, rhs), new_matching)
            },
            Rule::v_decimal => {
                (pair.as_str().parse::<f64>().ok().unwrap(), matching)
            },
            Rule::var => {
                let var = lexicon.intern("var", pair.as_str(), true);
                (matching.get(var).unwrap().text.parse::<f64>().ok().unwrap(), matching)
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
        "%" => lhs % rhs,
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
