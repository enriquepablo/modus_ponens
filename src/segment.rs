use std::hash::{Hash, Hasher};
use std::fmt;

use crate::constants;

#[derive(Debug, Clone)]
pub struct SynSegment {
    pub text: String,
    pub name: String,
    pub is_leaf: bool,
    pub is_var: bool,
}

impl SynSegment {
    pub fn new(name: String, text: String, is_leaf: bool) -> SynSegment {
        let is_var = name == constants::VAR_RULE_NAME;
        SynSegment {
            name, text,
            is_leaf, is_var,
        }
    }

    pub fn in_var_range(&self) -> bool {
        self.name.starts_with(constants::VAR_RANGE_PREFIX)
    }
}

impl fmt::Display for SynSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.text)
    }
}

impl PartialEq for SynSegment {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.text == other.text
    }
}

impl Eq for SynSegment {}

impl Hash for SynSegment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.text.hash(state);
        self.is_leaf.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    use crate::lexicon::Lexicon;

    #[test]
    fn make_segment() {
        let name = "rule-name".to_string();
        let text = "some text".to_string();
        let segm = SynSegment::new(name, text, true);
        assert_eq!(segm.name, "rule-name");
        assert_eq!(segm.text, "some text");
        assert_eq!(segm.is_leaf, true);
        assert_eq!(segm.is_var, false);
    }

    #[test]
    fn make_var() {
        let lexicon = Lexicon::new();
        let var = lexicon.make_var(0);
        assert_eq!(var.name, constants::VAR_RULE_NAME);
        assert_eq!(var.text, "<__X0>");
        assert_eq!(var.is_leaf, true);
        assert_eq!(var.is_var, true);
    }

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn segment_eq1() {
        let name = "rule-name";
        let text = "some text";
        let segm1 = SynSegment::new(name.to_string(), text.to_string(), true);
        let segm2 = SynSegment::new(name.to_string(), text.to_string(), true);
        assert!(segm1 == segm2);
        assert!(calculate_hash(&segm1) == calculate_hash(&segm2));
    }

    #[test]
    fn segment_eq2() {
        let name = "rule-name";
        let text = "some text";
        let segm1 = SynSegment::new(name.to_string(), text.to_string(), true);
        let segm2 = SynSegment::new(name.to_string(), text.to_string(), true);
        assert!(segm1 == segm2);
        assert!(calculate_hash(&segm1) == calculate_hash(&segm2));
    }

    #[test]
    fn segment_noteq1() {
        let name1 = "rule-name1";
        let name2 = "rule-name2";
        let text = "some text";
        let segm1 = SynSegment::new(name1.to_string(), text.to_string(), true);
        let segm2 = SynSegment::new(name2.to_string(), text.to_string(), true);
        assert!(segm1 != segm2);
        assert!(calculate_hash(&segm1) != calculate_hash(&segm2));
    }

    #[test]
    fn segment_noteq2() {
        let name = "rule-name".to_string();
        let text1 = "some text 1";
        let text2 = "some text 2";
        let segm1 = SynSegment::new(name.to_string(), text1.to_string(), true);
        let segm2 = SynSegment::new(name.to_string(), text2.to_string(), true);
        assert!(segm1 != segm2);
        assert!(calculate_hash(&segm1) != calculate_hash(&segm2));
    }

    #[test]
    fn segment_in_var_range() {
        let name = "v_rule-name".to_string();
        let text = "some text".to_string();
        let segm = SynSegment::new(name, text, true);
        assert!(segm.in_var_range());
    }

    #[test]
    fn segment_not_in_var_range() {
        let name = "rule-name".to_string();
        let text = "some text".to_string();
        let segm = SynSegment::new(name, text, true);
        assert!(!segm.in_var_range());
    }
}
