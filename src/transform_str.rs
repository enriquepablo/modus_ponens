use pest::Parser;
//use log::debug;

use crate::lexicon::Lexicon;
use crate::matching::MPMatching;
use crate::segment::MPSegment;

#[derive(Parser)]
#[grammar = "transform_str.pest"]
pub struct SParser;

pub struct TSParser<'a> {
    lexicon: &'a Lexicon,
}

impl<'a> TSParser<'a> {

    pub fn new(lexicon: &'a Lexicon) -> TSParser<'a> {
        TSParser {
            lexicon
        }
    }

    pub fn compile(&self, source: &str, matching: &MPMatching<'a>) -> &MPSegment {
        let parse_result = SParser::parse(Rule::expr, source);
        if parse_result.is_err() {
            panic!("These do not seem like transforms: \"{}\"\n\nerr: {}\n\nmatching: {:?}", source, parse_result.err().unwrap(), matching);
        }
        let pair = parse_result.ok().unwrap().next().unwrap();

        self.compile_expr(pair, matching)
    }

    fn compile_expr(&self, pair: pest::iterators::Pair<Rule>, matching: &MPMatching<'a>) -> &MPSegment {
        match pair.as_rule() {
            Rule::monadicExpr => {
                let mut pair = pair.into_inner();
                let op = pair.next().expect("cin");
                let termpair = pair.next().expect("sei");
                let term = self.compile_expr(termpair, matching);
                self.parse_monadic_op(op, term)
            },
            Rule::dyadicExpr => {
                let mut pair = pair.into_inner();
                let op = pair.next().expect("och");
                let fstpair = pair.next().expect("sie");
                let fst = self.compile_expr(fstpair, matching);
                let sndpair = pair.next().expect("nue");
                let snd = self.compile_expr(sndpair, matching);
                self.parse_dyadic_op(op, fst, snd)
            },
            Rule::triadicExpr => {
                let mut pair = pair.into_inner();
                let op = pair.next().expect("och");
                let fstpair = pair.next().expect("sie");
                let fst = self.compile_expr(fstpair, matching);
                let sndpair = pair.next().expect("nue");
                let snd = self.compile_expr(sndpair, matching);
                let trdpair = pair.next().expect("nue");
                let trd = self.compile_expr(trdpair, matching);
                self.parse_triadic_op(op, fst, snd, trd)
            },
            Rule::v_decimal => {
                self.lexicon.intern("v_decimal", pair.as_str(), true)
            },
            Rule::v_string => {
                self.lexicon.intern("v_string", pair.as_str(), true)
            },
            Rule::var => {
                let var = self.lexicon.intern("var", pair.as_str(), true);
                matching.get(var).expect("segment")
            },
            unknown_expr => panic!("Unexpected expression: {:?}", unknown_expr),
        }
    }

    fn parse_triadic_op(&self, op: pest::iterators::Pair<Rule>, fst: &MPSegment, snd: &MPSegment, trd: &MPSegment) -> &MPSegment {
        match op.as_str() {
            "substring" => {
                let fst_str = &fst.text;
                let snd_num = snd.text.as_str().parse::<usize>().ok().expect("die");
                let trd_num = trd.text.as_str().parse::<usize>().ok().expect("die");
                let substr = fst_str.chars().skip(snd_num).take(trd_num).collect();
                self.lexicon.intern_with_text(&fst.name, substr, true)
            },
            "replace" => {
                let fst_str = &fst.text;
                let snd_str = &snd.text;
                let trd_str = &trd.text;
                let result = fst_str.replace(snd_str, trd_str);
                self.lexicon.intern_with_text(&fst.name, result, true)
            },
            _ => panic!("Unexpected triadic operator: {}", op.as_str()),
        }
    }

    fn parse_dyadic_op(&self, op: pest::iterators::Pair<Rule>, fst: &MPSegment, snd: &MPSegment) -> &MPSegment {
        match op.as_str() {
            "index_of" => {
                let fst_str = &fst.text;
                let snd_str = &snd.text;
                let result = fst_str.find(snd_str);
                match result {
                    None => self.lexicon.intern("v_decimal", "-1", true),
                    Some(i) => self.lexicon.intern_with_text("v_decimal", format!("{}", i), true),
                }
            },
            "concat" => {
                let mut fst_str = fst.text.clone();
                fst_str.push_str(&snd.text);
                self.lexicon.intern_with_text(&fst.name, fst_str, true)
            },
            _ => panic!("Unexpected dyadic operator: {}", op.as_str()),
        }
    }

    fn parse_monadic_op(&self, op: pest::iterators::Pair<Rule>, term: &MPSegment) -> &MPSegment {
        match op.as_str() {
            "len" => {
                let result = format!("{}", term.text.len());
                self.lexicon.intern_with_text("v_decimal", result, true)
            },
            _ => panic!("Unexpected monadic operator: {}", op.as_str()),
        }
    }
}
