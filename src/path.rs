use std::hash::{Hash, Hasher};
use std::fmt;

use crate::segment::SynSegment;
use crate::matching::{SynMatching, get_or_key};

#[derive(Debug, Clone)]
pub struct SynPath {
    pub value: SynSegment,
    pub segments: Vec<SynSegment>,
    identity: Vec<String>,
}

impl<'a> SynPath {
    pub fn new(segments: Vec<SynSegment>) -> SynPath {
        let mut identity = Vec::new();
        let mut new_segments = Vec::new();
        for segment in segments {
            identity.push(segment.name.clone());
            new_segments.push(segment);
        }
        let value = new_segments.last().expect("no empty paths").clone();
        identity.push(value.text.clone());
        SynPath { value, segments: new_segments, identity, }
    }
    pub fn empty_root() -> SynPath {
        let segments = vec![SynSegment::new("root", "empty", false)];
        SynPath::new(segments)
    }
    pub fn len(&self) -> usize {
        self.segments.len()
    }
    pub fn is_var(&self) -> bool {
        self.value.is_var
    }
    pub fn is_leaf(&self) -> bool {
        self.value.is_leaf
    }
    pub fn starts_with(&self, path: &SynPath) -> bool {
        let lself = self.len();
        let lpath = path.len();
        lself >= lpath && &self.segments[0..lpath] == &path.segments[0..lpath]
    }
    pub fn sub_path(&self, lpath: usize) -> (SynPath, &SynSegment) {
        let new_segments = self.segments
                               .as_slice()
                               .into_iter()
                               .take(lpath)
                               .map(|s| s.clone())
                               .collect();
        (SynPath::new(new_segments), self.segments.last().unwrap())
    }
    pub fn paths_after(&self, paths: &'a [&'a SynPath], try_to_see: bool) -> &'a [&'a SynPath] {
        let mut seen = false;
        let mut path_starts_with_self: bool;
        let mut i = 0;
        let mut after = 0;
        for path in paths {
            path_starts_with_self = path.starts_with(&self);
            if path_starts_with_self {
                after = i;
            }
            println!("starts {} really with: {}", after, path);
            if try_to_see && !seen && path_starts_with_self {
                seen = true;
                println!("seen: {}", path);
            } else if (!try_to_see || seen) && (!path_starts_with_self || path.len() == self.len()) {
                after = i;
                println!("break: {}", after);
                break;
            }
            i += 1;
        }
        &paths[after..]
    }
    pub fn in_var_range(&self) -> bool {
        self.value.in_var_range()
    }

    pub fn substitute(&self, matching: &SynMatching) -> SynPath {
        let mut segments = self.segments.clone();
        let var = segments.pop().unwrap();
        let value = *matching.get(&var).unwrap();
        segments.push(value.clone());
        SynPath::new(segments)
    }

    pub fn substitute2(self, matching: &SynMatching) -> (SynPath, SynPath) {
        let SynPath { segments, .. } = self;
        let mut new_segment: SynSegment;
        let mut new_segments = vec![];
        let mut old_segments = vec![];
        for segment in segments {
            new_segment = get_or_key(matching, &segment);
            let is_new = new_segment != segment;
            new_segments.push(new_segment);
            old_segments.push(segment);
            if is_new {
                let new_path = SynPath::new(new_segments);
                let old_path = SynPath::new(old_segments);
                return (new_path, old_path);
            }
        }
        (SynPath::new(new_segments), SynPath::new(old_segments))
    }
}

impl<'a> fmt::Display for SynPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.value)
    }
}

impl<'a> PartialEq for SynPath {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl<'a> Eq for SynPath {}

impl<'a> Hash for SynPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::collections::hash_map::DefaultHasher;
    
    use crate::constants;

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn make_path_1() {
        let name = "rule-name".to_string();
        let text = "some text".to_string();
        let segm = SynSegment::new(&name, &text, true);
        let segms = vec![segm];
        let path = SynPath::new(segms);
        assert_eq!(path.identity[0], "rule-name");
        assert_eq!(path.identity[1], "some text");
        assert_eq!(path.segments[0].name, "rule-name");
        assert_eq!(path.segments[0].text, "some text");
        assert_eq!(path.len(), 1);
        assert_eq!(path.is_var(), false);
        assert_eq!(path.is_leaf(), true);
    }

    #[test]
    fn make_var_path_1() {
        let segm = SynSegment::make_var(0);
        let segms = vec![segm];
        let path = SynPath::new(segms);
        assert_eq!(path.identity[0], constants::VAR_RULE_NAME);
        assert_eq!(path.identity[1], "<__X0>");
        assert_eq!(path.len(), 1);
        assert_eq!(path.is_var(), true);
        assert_eq!(path.is_leaf(), true);
    }

    #[test]
    fn make_path_2() {
        let segm1 = SynSegment::new("rule-name1", "some text1", false);
        let segm2 = SynSegment::new("rule-name2", "some text2", true);
        let segms = vec![segm1, segm2];
        let path = SynPath::new(segms);
        assert_eq!(path.identity[0], "rule-name1");
        assert_eq!(path.identity[1], "rule-name2");
        assert_eq!(path.identity[2], "some text2");
        assert_eq!(path.segments[0].name, "rule-name1");
        assert_eq!(path.segments[0].text, "some text1");
        assert_eq!(path.segments[1].name, "rule-name2");
        assert_eq!(path.segments[1].text, "some text2");
        assert_eq!(path.len(), 2);
    }

    #[test]
    fn make_paths_1() {
        let segm11 = SynSegment::new("rule-name1", "some text1", false);
        let segm12 = SynSegment::new("rule-name2", "some text2", true);
        let segms1 = vec![segm11, segm12];
        let path1 = SynPath::new(segms1);
        let segm21 = SynSegment::new("rule-name1", "some text1", false);
        let segm22 = SynSegment::new("rule-name2", "some text2", true);
        let segms2 = vec![segm21, segm22];
        let path2 = SynPath::new(segms2);
        assert_eq!(&path1, &path2);
        assert_eq!(calculate_hash(&path1), calculate_hash(&path2));
    }

    #[test]
    fn make_paths_2() {
        let segm11 = SynSegment::new("rule-name1", "some text3", false);
        let segm12 = SynSegment::new("rule-name2", "some text2", true);
        let segms1 = vec![segm11, segm12];
        let path1 = SynPath::new(segms1);
        let segm21 = SynSegment::new("rule-name1", "some text1", false);
        let segm22 = SynSegment::new("rule-name2", "some text2", true);
        let segms2 = vec![segm21, segm22];
        let path2 = SynPath::new(segms2);
        assert_eq!(&path1, &path2);
        assert_eq!(calculate_hash(&path1), calculate_hash(&path2));
    }

    #[test]
    fn starts_with_path_1() {
        let segm11 = SynSegment::new("rule-name1", "some text1", false);
        let segm12 = SynSegment::new("rule-name2", "some text2", true);
        let segms1 = vec![segm11, segm12];
        let path1 = SynPath::new(segms1);
        let segm21 = SynSegment::new("rule-name1", "some text1", false);
        let segm22 = SynSegment::new("rule-name2", "some text2", true);
        let segms2 = vec![segm21, segm22];
        let path2 = SynPath::new(segms2);
        assert!(path1.starts_with(&path2));
    }

    #[test]
    fn starts_with_path_2() {
        let segm11 = SynSegment::new("rule-name1", "some text1", false);
        let segm12 = SynSegment::new("rule-name2", "some text2", true);
        let segms1 = vec![segm11, segm12];
        let path1 = SynPath::new(segms1);
        let segm21 = SynSegment::new("rule-name1", "some text1", false);
        let segm22 = SynSegment::new("rule-name2", "some text2", false);
        let segm23 = SynSegment::new("rule-name3", "some text3", true);
        let segms2 = vec![segm21, segm22, segm23];
        let path2 = SynPath::new(segms2);
        assert!(path2.starts_with(&path1));
        assert_ne!(path1.starts_with(&path2), true);
    }

    #[test]
    fn starts_with_path_3() {
        let segm11 = SynSegment::new("rule-name1", "some text1", false);
        let segm12 = SynSegment::new("rule-name2", "some text2", false);
        let segm13 = SynSegment::new("rule-name3", "some text3", true);
        let segms1 = vec![segm11, segm12, segm13];
        let path1 = SynPath::new(segms1);
        let segm21 = SynSegment::new("rule-name1", "some text1", false);
        let segm22 = SynSegment::new("rule-name2", "some text2", false);
        let segm23 = SynSegment::new("rule-name3", "some text3", false);
        let segm24 = SynSegment::new("rule-name4", "some text4", false);
        let segm25 = SynSegment::new("rule-name5", "some text5", true);
        let segms2 = vec![segm21, segm22, segm23, segm24, segm25];
        let path2 = SynPath::new(segms2);
        assert!(path2.starts_with(&path1));
        assert_ne!(path1.starts_with(&path2), true);
    }

    #[test]
    fn not_starts_with_path_1() {
        let segm11 = SynSegment::new("rule-name9", "some text1", false);
        let segm12 = SynSegment::new("rule-name2", "some text2", false);
        let segm13 = SynSegment::new("rule-name3", "some text3", true);
        let segms1 = vec![segm11, segm12, segm13];
        let path1 = SynPath::new(segms1);
        let segm21 = SynSegment::new("rule-name1", "some text1", false);
        let segm22 = SynSegment::new("rule-name2", "some text2", false);
        let segm23 = SynSegment::new("rule-name3", "some text3", false);
        let segm24 = SynSegment::new("rule-name4", "some text4", false);
        let segm25 = SynSegment::new("rule-name5", "some text5", true);
        let segms2 = vec![segm21, segm22, segm23, segm24, segm25];
        let path2 = SynPath::new(segms2);
        assert_ne!(path2.starts_with(&path1), true);
        assert_ne!(path1.starts_with(&path2), true);
    }

    #[test]
    fn not_starts_with_path_2() {
        let segm11 = SynSegment::new("rule-name1", "some text9", false);
        let segm12 = SynSegment::new("rule-name2", "some text2", false);
        let segm13 = SynSegment::new("rule-name3", "some text3", true);
        let segms1 = vec![segm11, segm12, segm13];
        let path1 = SynPath::new(segms1);
        let segm21 = SynSegment::new("rule-name1", "some text1", false);
        let segm22 = SynSegment::new("rule-name2", "some text2", false);
        let segm23 = SynSegment::new("rule-name3", "some text3", false);
        let segm24 = SynSegment::new("rule-name4", "some text4", false);
        let segm25 = SynSegment::new("rule-name5", "some text5", true);
        let segms2 = vec![segm21, segm22, segm23, segm24, segm25];
        let path2 = SynPath::new(segms2);
        assert_ne!(path2.starts_with(&path1), true);
        assert_ne!(path1.starts_with(&path2), true);
    }

    #[test]
    fn paths_after_1() {
        let segm11 = SynSegment::new("rule-name1", "some text1", false);
        let segms1 = vec![segm11];
        let path1 = SynPath::new(segms1);
        let segm21 = SynSegment::new("rule-name1", "some text1", false);
        let segm22 = SynSegment::new("rule-name2", "some text2", false);
        let segms2 = vec![segm21, segm22];
        let path2 = SynPath::new(segms2);
        let segm31 = SynSegment::new("rule-name1", "some text1", false);
        let segm32 = SynSegment::new("rule-name2", "some text2", false);
        let segm33 = SynSegment::new("rule-name3", "some text3", false);
        let segms3 = vec![segm31, segm32, segm33];
        let path3 = SynPath::new(segms3);
        let segm41 = SynSegment::new("rule-name1", "some text1", false);
        let segm43 = SynSegment::new("rule-name3", "some text3", false);
        let segm44 = SynSegment::new("rule-name4", "some text4", false);
        let segms4 = vec![segm41, segm43, segm44];
        let path4 = SynPath::new(segms4);
        let segm51 = SynSegment::new("rule-name1", "some text1", false);
        let segm53 = SynSegment::new("rule-name3", "some text3", false);
        let segm54 = SynSegment::new("rule-name4", "some text4", false);
        let segm55 = SynSegment::new("rule-name5", "some text5", true);
        let segms5 = vec![segm51, segm53, segm54, segm55];
        let path5 = SynPath::new(segms5);

        let paths = vec![&path1, &path2, &path3, &path4, &path5];

        let segm61 = SynSegment::new("rule-name1", "some text1", false);
        let segm62 = SynSegment::new("rule-name2", "some text2", false);
        let segms6 = vec![segm61, segm62];
        let path6 = SynPath::new(segms6);

        let paths_after1 = path6.paths_after(&paths, false);
        assert_eq!(paths_after1.len(), 5);

        let paths2 = vec![&path2, &path3, &path4, &path5];

        let paths_after2 = path6.paths_after(&paths2, false);
        assert_eq!(paths_after2.len(), 4);

        let paths_after3 = path6.paths_after(&paths, true);
        assert_eq!(paths_after3.len(), 2);

        let segm71 = SynSegment::new("rule-name1", "some text1", false);
        let segms7 = vec![segm71];
        let path7 = SynPath::new(segms7);

        let paths_after4 = path7.paths_after(&paths2, false);
        assert_eq!(paths_after4.len(), 1);

        let paths_after5 = path7.paths_after(&paths2, true);
        assert_eq!(paths_after5.len(), 1);
    }

    #[test]
    fn var_range_1() {
        let segm11 = SynSegment::new("rule-name1", "some text1", false);
        let segm12 = SynSegment::new("v_rule-name2", "some text2", true);
        let segms1 = vec![segm11, segm12];
        let path1 = SynPath::new(segms1);
        assert!(path1.in_var_range());
    }

    #[test]
    fn not_var_range_1() {
        let segm11 = SynSegment::new("rule-name1", "some text1", false);
        let segm12 = SynSegment::new("rule-name2", "some text2", true);
        let segms1 = vec![segm11, segm12];
        let path1 = SynPath::new(segms1);
        assert!(!path1.in_var_range());
    }

    #[test]
    fn substitute_1() {
        let segm11 = SynSegment::new("rule-name1", "some text1", false);
        let segm12 = SynSegment::new("rule-name2", "some text2", false);
        let segm13 = SynSegment::new("rule-name3", "some text3", true);
        let segms1 = vec![segm11, segm12, segm13];
        let path1 = SynPath::new(segms1);
        let segm23 = SynSegment::new("rule-name3", "some text3", false);
        let segm24 = SynSegment::new("rule-name4", "some text4", false);
        let mut matching: SynMatching = HashMap::new();
        matching.insert(&segm23, &segm24);

        let (new_path, old_path) = path1.substitute2(&matching);

        assert_eq!(new_path.value.name, "rule-name4");
        assert_eq!(old_path.value.name, "rule-name3");
    }

    #[test]
    fn substitute_2() {
        let segm11 = SynSegment::new("rule-name1", "some text1", false);
        let segm12 = SynSegment::new("rule-name2", "some text2", false);
        let segm13 = SynSegment::new("rule-name3", "some text3", true);
        let segms1 = vec![segm11, segm12, segm13];
        let path1 = SynPath::new(segms1);
        let segm23 = SynSegment::new("rule-name5", "some text3", false);
        let segm24 = SynSegment::new("rule-name4", "some text4", false);
        let mut matching: SynMatching = HashMap::new();
        matching.insert(&segm23, &segm24);

        let (new_path, old_path) = path1.substitute2(&matching);

        assert_eq!(new_path.value.name, "rule-name3");
        assert_eq!(old_path.value.name, "rule-name3");
    }
}
