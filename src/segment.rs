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

use std::hash::{Hash, Hasher};
use std::fmt;


#[derive(Debug, Clone)]
pub struct MPSegment {
    pub text: String,
    pub name: String,
    pub is_leaf: bool,
    pub is_var: bool,
    pub in_var_range: bool,
    pub is_empty: bool,
}

impl MPSegment {
    pub fn new(name: String, text: String, is_leaf: bool, is_var: bool, in_var_range: bool) -> MPSegment {
        let is_empty = text.trim().is_empty();
        MPSegment {
            name, text,
            is_leaf, is_var,
            in_var_range, is_empty,
        }
    }
}

impl fmt::Display for MPSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.text)
    }
}

impl PartialEq for MPSegment {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.text == other.text
    }
}

impl Eq for MPSegment {}

impl Hash for MPSegment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.text.hash(state);
        self.is_leaf.hash(state);
    }
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use std::collections::hash_map::DefaultHasher;
//
//    use crate::lexicon::Lexicon;
//
//    #[test]
//    fn make_segment() {
//        let name = "rule-name".to_string();
//        let text = "some text".to_string();
//        let segm = MPSegment::new(name, text, true);
//        assert_eq!(segm.name, "rule-name");
//        assert_eq!(segm.text, "some text");
//        assert_eq!(segm.is_leaf, true);
//        assert_eq!(segm.is_var, false);
//    }
//
//    #[test]
//    fn make_var() {
//        let lexicon = Lexicon::new();
//        let var = lexicon.make_var(0);
//        assert_eq!(var.name, constants::VAR_RULE_NAME);
//        assert_eq!(var.text, "<__X0>");
//        assert_eq!(var.is_leaf, true);
//        assert_eq!(var.is_var, true);
//    }
//
//    fn calculate_hash<T: Hash>(t: &T) -> u64 {
//        let mut s = DefaultHasher::new();
//        t.hash(&mut s);
//        s.finish()
//    }
//
//    #[test]
//    fn segment_eq1() {
//        let name = "rule-name";
//        let text = "some text";
//        let segm1 = MPSegment::new(name.to_string(), text.to_string(), true);
//        let segm2 = MPSegment::new(name.to_string(), text.to_string(), true);
//        assert!(segm1 == segm2);
//        assert!(calculate_hash(&segm1) == calculate_hash(&segm2));
//    }
//
//    #[test]
//    fn segment_eq2() {
//        let name = "rule-name";
//        let text = "some text";
//        let segm1 = MPSegment::new(name.to_string(), text.to_string(), true);
//        let segm2 = MPSegment::new(name.to_string(), text.to_string(), true);
//        assert!(segm1 == segm2);
//        assert!(calculate_hash(&segm1) == calculate_hash(&segm2));
//    }
//
//    #[test]
//    fn segment_noteq1() {
//        let name1 = "rule-name1";
//        let name2 = "rule-name2";
//        let text = "some text";
//        let segm1 = MPSegment::new(name1.to_string(), text.to_string(), true);
//        let segm2 = MPSegment::new(name2.to_string(), text.to_string(), true);
//        assert!(segm1 != segm2);
//        assert!(calculate_hash(&segm1) != calculate_hash(&segm2));
//    }
//
//    #[test]
//    fn segment_noteq2() {
//        let name = "rule-name".to_string();
//        let text1 = "some text 1";
//        let text2 = "some text 2";
//        let segm1 = MPSegment::new(name.to_string(), text1.to_string(), true);
//        let segm2 = MPSegment::new(name.to_string(), text2.to_string(), true);
//        assert!(segm1 != segm2);
//        assert!(calculate_hash(&segm1) != calculate_hash(&segm2));
//    }
//
//    #[test]
//    fn segment_in_var_range() {
//        let name = "v_rule-name".to_string();
//        let text = "some text".to_string();
//        let segm = MPSegment::new(name, text, true);
//        assert!(segm.in_var_range);
//    }
//
//    #[test]
//    fn segment_not_in_var_range() {
//        let name = "rule-name".to_string();
//        let text = "some text".to_string();
//        let segm = MPSegment::new(name, text, true);
//        assert!(!segm.in_var_range);
//    }
//}
//
