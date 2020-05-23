use pest::Parser;
//use log::debug;

use crate::lexicon::Lexicon;
use crate::matching::MPMatching;

#[derive(Parser)]
#[grammar = "condition.pest"]
pub struct CParser;

impl<'a> CParser {

    pub fn check_conditions(source: &str, matching: &MPMatching<'a>, lexicon: &'a Lexicon) -> bool {

        let parse_result = CParser::parse(Rule::conditions, source);
        if parse_result.is_err() {
            panic!("These do not seem like conditions: \"{}\"\n\nerr: {}\n\nmatching: {:?}", source, parse_result.err().unwrap(), matching);
        }
        let mut pairs = parse_result.ok().unwrap();

        for pair in pairs.next().unwrap().into_inner() {
            let mut exprpair = pair.into_inner();
            let t1pair = exprpair.next().expect("1st term");
            let val1 = CParser::compile_term(t1pair, matching, lexicon);

            let pred = exprpair.next().expect("the condition's pred").as_str();

            let t2pair = exprpair.next().expect("2st term");
            let val2 = CParser::compile_term(t2pair, matching, lexicon);

            let pass = eval_condition(val1, pred, val2);
            if !pass {
                return false;
            }
        }
        true
    }

    fn compile_term(pair: pest::iterators::Pair<Rule>, matching: &MPMatching<'a>, lexicon: &Lexicon) -> f64 {
        match pair.as_rule() {
            Rule::v_decimal => {
                pair.as_str().parse::<f64>().ok().expect("a number")
            },
            Rule::var => {
                let var = lexicon.intern("var", pair.as_str(), true);
                let number = matching.get(var).expect("number segment");
                let result = number.text.parse::<f64>();
                if result.is_err() {
                    panic!("This does not seem like a number: \"{}\"\n\nerr: {}\n\nmatching: {:?}", number.text, result.err().unwrap(), matching);
                }
                result.ok().unwrap()
            },
            unknown_expr => panic!("Unexpected expression: {:?}", unknown_expr),
        }
    }
}

fn eval_condition(lhs: f64, op: &str, rhs: f64) -> bool {
    match op {
        "==" => lhs == rhs,
        "!=" => lhs != rhs,
        "<" => lhs < rhs,
        ">" => lhs > rhs,
        "<=" => lhs <= rhs,
        ">=" => lhs >= rhs,
        _ => panic!("Unexpected dyadic operator: {}", op),
    }
}
