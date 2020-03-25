use std::{cell::RefCell, collections::HashSet, mem};
use std::clone::Clone;
use std::hash::{Hash, Hasher};
use std::fmt;


use crate::path::SynPath;



#[derive(Debug, Clone)]
pub struct Fact<'a> {
    pub text: String,
    pub paths: Vec<SynPath<'a>>,
}

impl<'a> fmt::Display for Fact<'a> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.text)
    }
}

impl<'a> Fact<'a> {
    fn new(text: String, paths: Vec<SynPath>) -> Fact {
        Fact { text, paths, }
    }
    pub fn initialize(text: String) -> Fact<'a> {
        Fact {
            text,
            paths: Vec::new(),
        }
    }
    pub fn initialize_str(text: &str) -> Fact<'a> {
        Fact {
            text: String::from(text),
            paths: Vec::new(),
        }
    }
    pub fn from_paths(paths: Vec<SynPath>) -> Fact {
        let text = paths.iter().map(|path| path.value.text.clone()).collect::<Vec<String>>().join("");
        Fact { text, paths, }
    }
    pub fn get_all_paths(&'a self) -> &'a [&'a SynPath] {
        let paths = &mut [];
        for (i, path) in self.paths.iter().enumerate() {
            if !path.value.text.trim().is_empty() {
                paths[i] = path;
            }
        }
        paths
    }
    pub fn get_leaf_paths(&self) -> Vec<&SynPath> {
        let mut paths = Vec::new();
        for path in self.paths.iter() {
            if path.is_leaf() && !path.value.text.trim().is_empty() {
                paths.push(path);
            }
        }
        paths
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

pub struct FLexicon<'a>(RefCell<HashSet<Fact<'a>>>);

impl<'a> FLexicon<'a> {
    pub fn new() -> Self {
        FLexicon(RefCell::new(HashSet::new()))
    }

    pub fn intern(&self, text: &'a str, paths: Vec<SynPath<'a>>) -> &Fact {
        let mut set = self.0.borrow_mut();
        
        let otext = String::from(text);
        let fact = Fact::new(otext, paths);

        if !set.contains(&fact) {
            set.insert(fact.clone());
        }

        let interned = set.get(&fact).unwrap();

        // TODO: Document the pre- and post-conditions that the code must
        // uphold to make this unsafe code valid instead of copying this
        // from Stack Overflow without reading it
        unsafe { mem::transmute(&interned) }
    }
    pub fn from_paths(&'a self, paths: Vec<SynPath<'a>>) -> &'a Fact<'a> {
        let mut set = self.0.borrow_mut();
        
        let text = paths.iter()
                        .filter(|path| path.is_leaf())
                        .map(|path| path.value.text.as_str())
                        .collect::<Vec<&str>>()
                        .join("");
        let fact = Fact::new(text, paths);

        if !set.contains(&fact) {
            set.insert(fact.clone());
        }

        let interned = set.get(&fact).unwrap();

        // TODO: Document the pre- and post-conditions that the code must
        // uphold to make this unsafe code valid instead of copying this
        // from Stack Overflow without reading it
        unsafe { mem::transmute(&interned) }
    }
    pub fn from_paths_and_boxed_string(&'a self, text: &str, paths: Vec<SynPath<'a>>) -> &'a Fact<'a> {
        let mut set = self.0.borrow_mut();
        
        let fact = Fact::new(String::from(text), paths);

        if !set.contains(&fact) {
            set.insert(fact.clone());
        }

        let interned = set.get(&fact).unwrap();

        // TODO: Document the pre- and post-conditions that the code must
        // uphold to make this unsafe code valid instead of copying this
        // from Stack Overflow without reading it
        unsafe { mem::transmute(&interned) }
    }

    pub fn complete_fact(&'a self, mut fact: Box<Fact<'a>>, paths: Vec<SynPath<'a>>) -> &'a Fact<'a> {
        let mut set = self.0.borrow_mut();

        let cloned = fact.clone();
        
        if !set.contains(&fact) {
            fact.paths = paths;
            set.insert(*fact);
        }

        let interned = set.get(&cloned).unwrap();

        // TODO: Document the pre- and post-conditions that the code must
        // uphold to make this unsafe code valid instead of copying this
        // from Stack Overflow without reading it
        unsafe { mem::transmute(&interned) }
    }
    pub fn intern_string(&self, text: String, paths: Vec<SynPath<'a>>) -> &Fact {
        let mut set = self.0.borrow_mut();
        
        let otext = String::from(text);
        let fact = Fact::new(otext, paths);

        if !set.contains(&fact) {
            set.insert(fact.clone());
        }

        let interned = set.get(&fact).unwrap();

        // TODO: Document the pre- and post-conditions that the code must
        // uphold to make this unsafe code valid instead of copying this
        // from Stack Overflow without reading it
        unsafe { mem::transmute(&interned) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment::SynSegment;

    #[test]
    fn fact_1() {
        let segm11 = SynSegment::new("rule-name1".to_string(), "(text)".to_string(), false);
        let segms1 = vec![&segm11];
        let path1 = SynPath::new(segms1);

        let segm21 = SynSegment::new("rule-name1".to_string(), "(text)".to_string(), false);
        let segm22 = SynSegment::new("rule-name2".to_string(), "(".to_string(), true);
        let segms2 = vec![&segm21, &segm22];
        let path2 = SynPath::new(segms2);

        let segm31 = SynSegment::new("rule-name1".to_string(), "(text)".to_string(), false);
        let segm32 = SynSegment::new("rule-name3".to_string(), "text".to_string(), true);
        let segms3 = vec![&segm31, &segm32];
        let path3 = SynPath::new(segms3);

        let segm41 = SynSegment::new("rule-name1".to_string(), "(text)".to_string(), false);
        let segm42 = SynSegment::new("rule-name4".to_string(), ")".to_string(), true);
        let segms4 = vec![&segm41, &segm42];
        let path4 = SynPath::new(segms4);

        let paths = vec![path1, path2, path3, path4];
        let fact = Fact::from_paths(paths);

        assert_eq!(fact.get_all_paths().len(), 4);
        assert_eq!(fact.get_leaf_paths().len(), 3);
    }

    #[test]
    fn fact_2() {
        let segm11 = SynSegment::new("rule-name1".to_string(), "(text )".to_string(), false);
        let segms1 = vec![&segm11];
        let path1 = SynPath::new(segms1);

        let segm21 = SynSegment::new("rule-name1".to_string(), "(text )".to_string(), false);
        let segm22 = SynSegment::new("rule-name2".to_string(), "(".to_string(), true);
        let segms2 = vec![&segm21, &segm22];
        let path2 = SynPath::new(segms2);

        let segm31 = SynSegment::new("rule-name1".to_string(), "(text )".to_string(), false);
        let segm32 = SynSegment::new("rule-name3".to_string(), "text".to_string(), true);
        let segms3 = vec![&segm31, &segm32];
        let path3 = SynPath::new(segms3);

        let segm41 = SynSegment::new("rule-name1".to_string(), "(text )".to_string(), false);
        let segm42 = SynSegment::new("rule-name4".to_string(), " ".to_string(), true);
        let segms4 = vec![&segm41, &segm42];
        let path4 = SynPath::new(segms4);

        let segm51 = SynSegment::new("rule-name1".to_string(), "(text )".to_string(), false);
        let segm52 = SynSegment::new("rule-name5".to_string(), ")".to_string(), true);
        let segms5 = vec![&segm51, &segm52];
        let path5 = SynPath::new(segms5);

        let paths = vec![path1, path2, path3, path4, path5];
        let fact = Fact::from_paths(paths);

        assert_eq!(fact.get_all_paths().len(), 4);
        assert_eq!(fact.get_leaf_paths().len(), 3);
    }
}
