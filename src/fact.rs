use std::{cell::RefCell, collections::HashSet, mem};
use std::clone::Clone;
use std::hash::{Hash, Hasher};
use std::fmt;


use crate::path::MPPath;



#[derive(Debug, Clone)]
pub struct Fact<'a> {
    pub text: &'a str,
    pub paths: Vec<MPPath<'a>>,
}

impl<'a> fmt::Display for Fact<'a> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.text)
    }
}

impl<'a> Fact<'a> {
    fn new(text: &'a str, paths: Vec<MPPath<'a>>) -> Fact<'a> {
        Fact { text, paths }
    }
}


impl<'a> PartialEq for Fact<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

impl<'a> Eq for Fact<'a> {}

impl<'a> Hash for Fact<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
    }
}

pub struct FLexicon<'a>(RefCell<HashSet<Box<Fact<'a>>>>);

impl<'a> FLexicon<'a> {
    pub fn new() -> Self {
        FLexicon(RefCell::new(HashSet::with_capacity(20)))
    }

    pub fn from_paths(&'a self, paths: Vec<MPPath<'a>>) -> &'a Fact<'a> {
        let mut set = self.0.borrow_mut();
        
        let text = paths.iter()
                        .filter(|path| path.value.is_leaf)
                        .map(|path| path.value.text.as_str())
                        .collect::<Vec<&str>>()
                        .join("");

        let stext = Box::leak(text.into_boxed_str());

        let interned = set.get_or_insert(Box::new(Fact::new(stext, paths))).as_ref();

        unsafe { mem::transmute(interned) }
    }
    pub fn from_paths_and_string(&'a self, text: &'a str, paths: Vec<MPPath<'a>>) -> &'a Fact<'a> {
        let mut set = self.0.borrow_mut();
        
        let interned = set.get_or_insert(Box::new(Fact::new(text, paths))).as_ref();

        unsafe { mem::transmute(interned) }
    }
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::segment::MPSegment;
//
////    #[test]
////    fn fact_1() {
////        let segm11 = MPSegment::new("rule-name1".to_string(), "(text)".to_string(), false);
////        let segms1 = vec![&segm11];
////        let path1 = MPPath::new(segms1);
////
////        let segm21 = MPSegment::new("rule-name1".to_string(), "(text)".to_string(), false);
////        let segm22 = MPSegment::new("rule-name2".to_string(), "(".to_string(), true);
////        let segms2 = vec![&segm21, &segm22];
////        let path2 = MPPath::new(segms2);
////
////        let segm31 = MPSegment::new("rule-name1".to_string(), "(text)".to_string(), false);
////        let segm32 = MPSegment::new("rule-name3".to_string(), "text".to_string(), true);
////        let segms3 = vec![&segm31, &segm32];
////        let path3 = MPPath::new(segms3);
////
////        let segm41 = MPSegment::new("rule-name1".to_string(), "(text)".to_string(), false);
////        let segm42 = MPSegment::new("rule-name4".to_string(), ")".to_string(), true);
////        let segms4 = vec![&segm41, &segm42];
////        let path4 = MPPath::new(segms4);
////
////        let paths = vec![path1, path2, path3, path4];
////        //let fact = Fact::from_paths(paths);
////
//////        assert_eq!(fact.get_all_paths().len(), 4);
//////        assert_eq!(fact.get_leaf_paths().len(), 3);
////    }
////
////    #[test]
////    fn fact_2() {
////        let segm11 = MPSegment::new("rule-name1".to_string(), "(text )".to_string(), false);
////        let segms1 = vec![&segm11];
////        let path1 = MPPath::new(segms1);
////
////        let segm21 = MPSegment::new("rule-name1".to_string(), "(text )".to_string(), false);
////        let segm22 = MPSegment::new("rule-name2".to_string(), "(".to_string(), true);
////        let segms2 = vec![&segm21, &segm22];
////        let path2 = MPPath::new(segms2);
////
////        let segm31 = MPSegment::new("rule-name1".to_string(), "(text )".to_string(), false);
////        let segm32 = MPSegment::new("rule-name3".to_string(), "text".to_string(), true);
////        let segms3 = vec![&segm31, &segm32];
////        let path3 = MPPath::new(segms3);
////
////        let segm41 = MPSegment::new("rule-name1".to_string(), "(text )".to_string(), false);
////        let segm42 = MPSegment::new("rule-name4".to_string(), " ".to_string(), true);
////        let segms4 = vec![&segm41, &segm42];
////        let path4 = MPPath::new(segms4);
////
////        let segm51 = MPSegment::new("rule-name1".to_string(), "(text )".to_string(), false);
////        let segm52 = MPSegment::new("rule-name5".to_string(), ")".to_string(), true);
////        let segms5 = vec![&segm51, &segm52];
////        let path5 = MPPath::new(segms5);
////
////        let paths = vec![path1, path2, path3, path4, path5];
//        //let fact = Fact::from_paths(paths);
//
////        assert_eq!(fact.get_all_paths().len(), 4);
////        assert_eq!(fact.get_leaf_paths().len(), 3);
////    }
//}
//