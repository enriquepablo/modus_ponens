use pest::Parser;

use crate::lexicon::Lexicon;
use crate::matching::MPMatching;
use crate::segment::MPSegment;

#[derive(Parser)]
#[grammar = "transform.pest"]
pub struct TParser;


pub fn process_transforms(source: &str, mut matching: MPMatching, lexicon: &Lexicon) -> MPMatching {
    let mut var: &MPSegment;
    let mut val: &MPSegment;

    let pairs = TParser::parse(Rule::transforms, source).ok().unwrap();
    for pair in pairs {
        match pair.as_rule() {
            Rule::var => {
                var = lexicon.intern("var", pair.as_str(), true);
            }
            Rule::expr => {
                let (new_val, new_matching) = compile_expr(pair, matching, lexicon);
                val = lexicon.intern("decimal", new_val.as_str(), true);
                matching = new_matching;
            }
            _ => {}
        }
        matching.insert(var, val);
    }

    matching
}

fn compile_expr(pair: pest::iterators::Pair<Rule>, matching: MPMatching, lexicon: &Lexicon) -> (f64, MPMatching) {
    match pair.as_rule() {
        Rule::expr => {
            compile_expr(pair.into_inner().next().unwrap(), matching, lexicon)
        },
        Rule::monadicExpr => {
            let mut pair = pair.into_inner();
            let op = pair.next().unwrap();
            let termpair = pair.next().unwrap();
            let (term, new_matching) = compile_expr(termpair, matching, lexicon);
            (parse_monadic_op(op, term), matching)
        },
        Rule::dyadicExpr => {
            let mut pair = pair.into_inner();
            let lhspair = pair.next().unwrap();
            let (lhs, new_matching) = compile_expr(lhspair, matching, lexicon);
            let op = pair.next().unwrap();
            let rhspair = pair.next().unwrap();
            let (rhs, matching) = compile_expr(rhspair, matching, lexicon);
            (parse_dyadic_op(op, lhs, rhs), matching)
        },
        Rule::decimal => {
            (pair.as_str() as f64, matching)
        },
        Rule::var => {
            let var = lexicon.intern("var", pair.as_str(), true);
            (matching.get(var).unwrap().text as f64, matching)
        },
        unknown_expr => panic!("Unexpected expression: {:?}", unknown_expr),
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
