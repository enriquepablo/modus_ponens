use std::{cell::RefCell, collections::{ HashSet, HashMap }, mem};

use crate::constants;
use crate::segment::MPSegment;
use crate::path::MPPath;



pub struct Lexicon {
    segments: RefCell<HashSet<Box<MPSegment>>>,
    names: RefCell<HashMap<String, (bool, bool)>>,
}

impl Lexicon {
    pub fn new() -> Self {
        Lexicon { 
            segments: RefCell::new(HashSet::new()),
            names: RefCell::new(HashMap::new()),
        }
    }

    pub fn intern(&self, name: &str, text: &str, is_leaf: bool) -> &MPSegment {
        let (is_var, in_var_range, new) = match self.names.borrow().get(name) {
            Some((is_var, in_var_range)) => (*is_var, *in_var_range, false),
            None => {
                let is_var = name == constants::VAR_RULE_NAME;
                let in_var_range = name.starts_with(constants::VAR_RANGE_PREFIX);
                (is_var, in_var_range, true)
            }
        };
        if new {
            self.names.borrow_mut().insert(name.to_string(), (is_var, in_var_range));
        }
        let mut set = self.segments.borrow_mut();
        let interned = set.get_or_insert(Box::new(MPSegment::new(name.to_string(),
                                                                                  text.to_string(),
                                                                                  is_leaf,
                                                                                  is_var,
                                                                                  in_var_range)));

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
