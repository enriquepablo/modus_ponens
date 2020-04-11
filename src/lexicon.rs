use std::{cell::RefCell, collections::HashSet, mem};

use crate::segment::SynSegment;
use crate::path::SynPath;



pub struct Lexicon(RefCell<HashSet<SynSegment>>);

impl Lexicon {
    pub fn new() -> Self {
        Lexicon(RefCell::new(HashSet::new()))
    }

    pub fn intern(&self, name: &str, text: &str, is_leaf: bool) -> &SynSegment {
        let mut set = self.0.borrow_mut();
        let value = SynSegment::new(name.to_string(), text.to_string(), is_leaf);

        if !set.contains(&value) {
            set.insert(value.clone());
        }

        let interned = set.get(&value).unwrap();

        // TODO: Document the pre- and post-conditions that the code must
        // uphold to make this unsafe code valid instead of copying this
        // from Stack Overflow without reading it
        unsafe { mem::transmute(interned) }
    }

    pub fn make_var(&self, n: usize) -> &SynSegment {
        let text = format!("<__X{}>", &n);
        self.intern(&text, "var", true)
    }
    pub fn empty_path(&self) -> SynPath {
        let root = self.intern("root", "empty", false);
        let segments = vec![root];
        SynPath::new(segments)
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
