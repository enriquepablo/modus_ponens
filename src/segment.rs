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
    pub fn new(name: &str, text: &str, is_leaf: bool) -> SynSegment {
        let is_var = name == constants::VAR_RULE_NAME.to_string();
        SynSegment {
            name: name.to_string(),
            text: text.to_string(),
            is_leaf, is_var,
        }
    }

    pub fn make_var(n: usize) -> SynSegment {
        let text = format!("<__X{}>", &n);
        SynSegment::new(constants::VAR_RULE_NAME, &text, true)
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn make_segment() {
        let name = "rule-name";
        let text = "some text";
        let segm = SynSegment::new(name, text, true);
        assert_eq!(segm.name, name);
        assert_eq!(segm.text, text);
        assert_eq!(segm.is_leaf, true);
        assert_eq!(segm.is_var, false);
    }

    #[test]
    fn make_var() {
        let var = SynSegment::make_var(0);
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
        let segm1 = SynSegment::new(name, text, true);
        let segm2 = SynSegment::new(name, text, true);
        assert!(segm1 == segm2);
        assert!(calculate_hash(&segm1) == calculate_hash(&segm2));
    }

    #[test]
    fn segment_eq2() {
        let name = "rule-name";
        let text = "some text";
        let segm1 = SynSegment::new(name, text, true);
        let segm2 = SynSegment::new(name, text, true);
        assert!(segm1 == segm2);
        assert!(calculate_hash(&segm1) == calculate_hash(&segm2));
    }

    #[test]
    fn segment_noteq1() {
        let name1 = "rule-name1";
        let name2 = "rule-name2";
        let text = "some text";
        let segm1 = SynSegment::new(name1, text, true);
        let segm2 = SynSegment::new(name2, text, true);
        assert!(segm1 != segm2);
        assert!(calculate_hash(&segm1) != calculate_hash(&segm2));
    }

    #[test]
    fn segment_noteq2() {
        let name = "rule-name";
        let text1 = "some text 1";
        let text2 = "some text 2";
        let segm1 = SynSegment::new(name, text1, true);
        let segm2 = SynSegment::new(name, text2, true);
        assert!(segm1 != segm2);
        assert!(calculate_hash(&segm1) != calculate_hash(&segm2));
    }

    #[test]
    fn segment_in_var_range() {
        let name = "v_rule-name";
        let text = "some text";
        let segm = SynSegment::new(name, text, true);
        assert!(segm.in_var_range());
    }

    #[test]
    fn segment_not_in_var_range() {
        let name = "rule-name";
        let text = "some text";
        let segm = SynSegment::new(name, text, true);
        assert!(!segm.in_var_range());
    }
}
