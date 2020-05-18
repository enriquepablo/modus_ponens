// Copyright (c) 2020 by Enrique Pérez Arnaud <enrique at cazalla.net>    
//    
// This file is part of the modus_ponens project.    
// http://www.modus_ponens.net    
//    
// The modus_ponens project is free software: you can redistribute it and/or modify    
// it under the terms of the GNU General Public License as published by    
// the Free Software Foundation, either version 3 of the License, or    
// (at your option) any later version.    
//    
// The modus_ponens project is distributed in the hope that it will be useful,    
// but WITHOUT ANY WARRANTY; without even the implied warranty of    
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the    
// GNU General Public License for more details.    
//    
// You should have received a copy of the GNU General Public License    
// along with any part of the modus_ponens project.    
// If not, see <http://www.gnu.org/licenses/>.

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
