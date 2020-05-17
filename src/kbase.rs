
use crate::matching::MPMatching;

pub trait KBase<'a> {
    fn tell(&'a self, knowledge: &'a str);
    fn ask(&'a self, knowledge: &'a str) -> Vec<MPMatching<'a>>;
}

pub trait KBGen<'a> {
    type Output: KBase<'a>;
    fn gen_kb() -> Self::Output;
}
