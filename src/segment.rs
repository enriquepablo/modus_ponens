use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

mod constants;

#[derive(Debug)]
struct Segment {
    text: String,
    name: String,
    start: usize,
    end: usize,
    is_leaf: bool,
    is_var: bool,
}

impl Segment {
    fn make_segment(name: &str, text: &str, start: usize, is_leaf: bool) -> Segment {
        let end = start + text.len();
        let is_var = name == constants::VAR_RULE_NAME.to_string();
        Segment {
            name: name.to_string(),
            text: text.to_string(),
            start: start,
            end: end,
            is_leaf: is_leaf,
            is_var: is_var,
        }
    }

    fn make_var(n: u16) -> Segment {
        let text = format!("<__X{}>", &n);
        Segment::make_segment(constants::VAR_RULE_NAME, &text, 0, true)
    }
}

impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.text == other.text
    }
}

impl Eq for Segment {}

impl Hash for Segment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.text.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_segment() {
        let name = "rule-name".to_string();
        let text = "some text".to_string();
        let segm = Segment::make_segment(&name, &text, 0, true);
        assert_eq!(segm.name, name);
        assert_eq!(segm.text, text);
        assert_eq!(segm.start, 0);
        assert_eq!(segm.end, 9);
        assert_eq!(segm.is_leaf, true);
        assert_eq!(segm.is_var, false);
    }

    #[test]
    fn make_var() {
        let var = Segment::make_var(0);
        assert_eq!(var.name, constants::VAR_RULE_NAME);
        assert_eq!(var.text, "<__X0>");
        assert_eq!(var.start, 0);
        assert_eq!(var.end, 6);
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
        let name = "rule-name".to_string();
        let text = "some text".to_string();
        let segm1 = Segment::make_segment(&name, &text, 0, true);
        let segm2 = Segment::make_segment(&name, &text, 0, true);
        assert!(segm1 == segm2);
        assert!(calculate_hash(&segm1) == calculate_hash(&segm2));
    }

    #[test]
    fn segment_eq2() {
        let name = "rule-name".to_string();
        let text = "some text".to_string();
        let segm1 = Segment::make_segment(&name, &text, 2, true);
        let segm2 = Segment::make_segment(&name, &text, 0, true);
        assert!(segm1 == segm2);
        assert!(calculate_hash(&segm1) == calculate_hash(&segm2));
    }

    #[test]
    fn segment_noteq1() {
        let name1 = "rule-name1".to_string();
        let name2 = "rule-name2".to_string();
        let text = "some text".to_string();
        let segm1 = Segment::make_segment(&name1, &text, 0, true);
        let segm2 = Segment::make_segment(&name2, &text, 0, true);
        assert!(segm1 != segm2);
        assert!(calculate_hash(&segm1) != calculate_hash(&segm2));
    }

    #[test]
    fn segment_noteq2() {
        let name = "rule-name".to_string();
        let text1 = "some text 1".to_string();
        let text2 = "some text 2".to_string();
        let segm1 = Segment::make_segment(&name, &text1, 0, true);
        let segm2 = Segment::make_segment(&name, &text2, 0, true);
        assert!(segm1 != segm2);
        assert!(calculate_hash(&segm1) != calculate_hash(&segm2));
    }
}
