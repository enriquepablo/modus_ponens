use std::{cell::RefCell, collections::HashSet, mem};

use crate::segment::MPSegment;
use crate::path::MPPath;



pub struct Lexicon(RefCell<HashSet<Box<MPSegment>>>);

impl Lexicon {
    pub fn new() -> Self {
        Lexicon(RefCell::new(HashSet::new()))
    }

    pub fn intern(&self, name: &str, text: &str, is_leaf: bool) -> &MPSegment {
        let mut set = self.0.borrow_mut();
        let interned = set.get_or_insert(Box::new(MPSegment::new(name.to_string(), text.to_string(), is_leaf)));

        unsafe { mem::transmute(interned.as_ref()) }
    }

    pub fn make_var(&self, n: usize) -> &MPSegment {
        let text = format!("<__X{}>", &n);
        self.intern("var", &text, true)
    }
    pub fn empty_path(&self) -> MPPath {
        let root = self.intern("fact", "0", false);
        let segments = vec![root];
        MPPath::new(segments)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let lexicon = Lexicon::new();
        let lex1 = lexicon.intern("name1", "text1", true);
        let _ = lexicon.intern("name2", "text2", true);
        let lex3 = lexicon.intern("name1", "text1", true);
        assert_eq!(lex1.name.as_ptr(), lex3.name.as_ptr());
    }
}
