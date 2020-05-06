use std::hash::{Hash, Hasher};
use std::fmt;

use crate::segment::SynSegment;
use crate::matching::{ SynMatching, get_or_key, get_or_key_owning };

#[derive(Debug, Clone)]
pub struct SynPath<'a> {
    pub value: &'a SynSegment,
    pub segments: Vec<&'a SynSegment>,
    identity: String,
}

impl<'a> SynPath<'a> {
    pub fn new(segments: Vec<&'a SynSegment>) -> SynPath {
        let value = *segments.last().expect("no empty paths");
        let mut identity = segments.iter()
                                           .map(|segment| &*segment.name)
                                           .collect::<Vec<&str>>()
                                           .concat();
        identity.push_str(&value.text);
        SynPath { value, segments, identity }
    }
    pub fn len(&self) -> usize {
        self.segments.len()
    }
    pub fn starts_with(&self, path: &SynPath) -> bool {
        let lself = self.len();
        let lpath = path.len();
        lself >= lpath && &self.segments[0..lpath] == &path.segments[0..lpath]
    }
    pub fn starts_with_slice(&self, path_slice: &'a [&'a SynSegment]) -> bool {
        let lself = self.len();
        let lpath = path_slice.len();
        lself >= lpath && self.segments[0..lpath] == path_slice[0..lpath]
    }
    pub fn sub_path(&'a self, lpath: usize) -> SynPath<'a> {
        let new_segments = &self.segments[0..lpath];
        SynPath::new(new_segments.to_vec())
    }
    pub fn sub_slice(&'a self, lpath: usize) -> (&'a [&'a SynSegment], &'a SynSegment) {
        let segments = &self.segments[0..lpath];
        (segments, segments.last().expect("no empty paths"))
    }

    pub fn paths_after(&'a self, paths: &'a [SynPath]) -> usize {
        let mut seen = false;
        let mut path_starts_with_self: bool;
        let mut i = 0;
        for path in paths {
            if path.value.is_empty {
                i += 1;
                continue;
            }
            path_starts_with_self = path.starts_with(&self);
            if path_starts_with_self {
                seen = true;
            } else if seen {
                break;
            }
            i += 1;
        }
        i as usize
    }


    pub fn paths_after_slice(path_slice: &'a [&'a SynSegment], paths: &'a [SynPath<'a>]) -> &'a [SynPath<'a>] {
        let mut i: u16 = 0;
        for path in paths {
            if path.value.is_empty || !path.value.is_leaf {
                i += 1;
                continue;
            }
            if !path.starts_with_slice(path_slice) {
                break;
            }
            i += 1;
        }
        &paths[i as usize..]
    }

    pub fn substitute(&'a self, matching: &'a SynMatching) -> (SynPath, Option<SynPath>) {
        let mut new_segments = Vec::with_capacity(self.segments.len());
        let mut old_segments = Vec::with_capacity(self.segments.len());
        let mut is_new = false;
        for segment in self.segments.iter() {
            let new_segment = get_or_key(&matching, &segment);
            is_new = &new_segment != segment;
            new_segments.push(new_segment);
            old_segments.push(*segment);
            if is_new {
                break;
            }
        }
        if is_new {
            new_segments.shrink_to_fit();
            old_segments.shrink_to_fit();
            let new_path = SynPath::new(new_segments);
            let old_path = SynPath::new(old_segments);
            (new_path, Some(old_path))
        } else {
            (SynPath::new(new_segments), None)
        }
    }

    pub fn substitute_owning(&'a self, matching: SynMatching<'a>) -> (SynPath, Option<SynPath>) {
        let mut new_segments = Vec::with_capacity(self.segments.len());
        let mut old_segments = Vec::with_capacity(self.segments.len());
        let mut is_new = false;
        for segment in self.segments.iter() {
            let new_segment = get_or_key_owning(matching.clone(), &segment);
            is_new = &new_segment != segment;
            new_segments.push(new_segment);
            old_segments.push(*segment);
            if is_new {
                break;
            }
        }
        if is_new {
            new_segments.shrink_to_fit();
            old_segments.shrink_to_fit();
            let new_path = SynPath::new(new_segments);
            let old_path = SynPath::new(old_segments);
            (new_path, Some(old_path))
        } else {
            (SynPath::new(new_segments), None)
        }
    }

    pub fn substitute_paths(paths: &'a [SynPath], matching: &'a SynMatching) -> Vec<SynPath<'a>> {
        let mut new_paths: Vec<SynPath> = Vec::with_capacity(paths.len());
        let mut old_paths: Vec<SynPath> = Vec::with_capacity(paths.len());
        for path in paths {
            let mut seen = false;
            for opath in old_paths.iter() {
                if path.len() > opath.len() && path.starts_with(opath) {
                    seen = true;
                    break;
                }
            }
            if !seen {
                let (new_path, old_path) = path.substitute(&matching);
                if old_path.is_some() {
                    old_paths.push(old_path.unwrap());
                    new_paths.push(new_path);
                } else if new_path.value.is_leaf {
                    new_paths.push(new_path);
                }
            }
        }
        new_paths.shrink_to_fit();
        new_paths
    }

    pub fn substitute_paths_owning(paths: &'a [SynPath], matching: SynMatching<'a>) -> Vec<SynPath<'a>> {
        let mut new_paths: Vec<SynPath> = Vec::with_capacity(paths.len());
        let mut old_paths: Vec<SynPath> = Vec::with_capacity(paths.len());
        for path in paths {
            let mut seen = false;
            for opath in old_paths.iter() {
                if path.starts_with(opath) && path.len() != opath.len() {
                    seen = true;
                    break;
                }
            }
            if !seen {
                let (new_path, old_path) = path.substitute_owning(matching.clone());
                if old_path.is_some() {
                    old_paths.push(old_path.unwrap());
                    new_paths.push(new_path);
                } else if new_path.value.is_leaf {
                    new_paths.push(new_path);
                }
            }
        }
        new_paths.shrink_to_fit();
        new_paths
    }
}


impl<'a> fmt::Display for SynPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.value)
    }
}

impl<'a> PartialEq for SynPath<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl<'a> Eq for SynPath<'_> {}

impl<'a> Hash for SynPath<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::collections::hash_map::DefaultHasher;
    
    //use crate::constants;
    //use crate::parser::Grammar;

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

//    #[test]
//    fn make_path_1() {
//        let name = "rule-name".to_string();
//        let text = "some text".to_string();
//        let segm = SynSegment::new(name, text, true);
//        let segms = vec![&segm];
//        let path = SynPath::new(segms);
//        assert_eq!(path.identity.split(" ").as_slice()[0], "rule-name".to_string());
//        assert_eq!(path.identity.split()[1], "some text".to_string());
//        assert_eq!(path.segments[0].name, "rule-name".to_string());
//        assert_eq!(path.segments[0].text, "some text".to_string());
//        assert_eq!(path.len(), 1);
//        assert_eq!(path.value.is_var, false);
//        assert_eq!(path.value.is_leaf, true);
//    }
//
//    #[test]
//    fn make_var_path_1() {
//        let grammar = Grammar::new();
//        let segm = grammar.lexicon.make_var(0);
//        let segms = vec![segm];
//        let path = SynPath::new(segms);
//        assert_eq!(path.identity[0], constants::VAR_RULE_NAME);
//        assert_eq!(path.identity[1], "<__X0>");
//        assert_eq!(path.len(), 1);
//        assert_eq!(path.value.is_var, true);
//        assert_eq!(path.value.is_leaf, true);
//    }
//
//    #[test]
//    fn make_path_2() {
//        let segm1 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
//        let segm2 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), true);
//        let segms = vec![&segm1, &segm2];
//        let path = SynPath::new(segms);
//        assert_eq!(path.identity[0], "rule-name1");
//        assert_eq!(path.identity[1], "rule-name2");
//        assert_eq!(path.identity[2], "some text2");
//        assert_eq!(path.segments[0].name, "rule-name1");
//        assert_eq!(path.segments[0].text, "some text1");
//        assert_eq!(path.segments[1].name, "rule-name2");
//        assert_eq!(path.segments[1].text, "some text2");
//        assert_eq!(path.len(), 2);
//    }

    #[test]
    fn make_paths_1() {
        let segm11 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
        let segm12 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), true);
        let segms1 = vec![&segm11, &segm12];
        let path1 = SynPath::new(segms1);
        let segm21 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
        let segm22 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), true);
        let segms2 = vec![&segm21, &segm22];
        let path2 = SynPath::new(segms2);
        assert_eq!(&path1, &path2);
        assert_eq!(calculate_hash(&path1), calculate_hash(&path2));
    }

    #[test]
    fn make_paths_2() {
        let segm11 = SynSegment::new("rule-name1".to_string(), "some text3".to_string(), false);
        let segm12 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), true);
        let segms1 = vec![&segm11, &segm12];
        let path1 = SynPath::new(segms1);
        let segm21 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
        let segm22 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), true);
        let segms2 = vec![&segm21, &segm22];
        let path2 = SynPath::new(segms2);
        assert_eq!(&path1, &path2);
        assert_eq!(calculate_hash(&path1), calculate_hash(&path2));
    }

    #[test]
    fn starts_with_path_1() {
        let segm11 = SynSegment::new("rule-name1".to_string(), "some text3".to_string(), false);
        let segm12 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), true);
        let segms1 = vec![&segm11, &segm12];
        let path1 = SynPath::new(segms1);
        let segm21 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
        let segm22 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), true);
        let segms2 = vec![&segm21, &segm22];
        let path2 = SynPath::new(segms2);
        assert!(path1.starts_with(&path2));
    }

    #[test]
    fn starts_with_path_2() {
        let segm11 = SynSegment::new("rule-name1".to_string(), "some text3".to_string(), false);
        let segm12 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), true);
        let segms1 = vec![&segm11, &segm12];
        let path1 = SynPath::new(segms1);
        let segm21 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
        let segm22 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), false);
        let segm23 = SynSegment::new("rule-name3".to_string(), "some text3".to_string(), true);
        let segms2 = vec![&segm21, &segm22, &segm23];
        let path2 = SynPath::new(segms2);
        assert!(path2.starts_with(&path1));
        assert_ne!(path1.starts_with(&path2), true);
    }

    #[test]
    fn starts_with_path_3() {
        let segm11 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
        let segm12 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), true);
        let segm13 = SynSegment::new("rule-name3".to_string(), "some text3".to_string(), true);
        let segms1 = vec![&segm11, &segm12, &segm13];
        let path1 = SynPath::new(segms1);
        let segm21 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
        let segm22 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), false);
        let segm23 = SynSegment::new("rule-name3".to_string(), "some text3".to_string(), false);
        let segm24 = SynSegment::new("rule-name4".to_string(), "some text4".to_string(), false);
        let segm25 = SynSegment::new("rule-name5".to_string(), "some text5".to_string(), true);
        let segms2 = vec![&segm21, &segm22, &segm23, &segm24, &segm25];
        let path2 = SynPath::new(segms2);
        assert!(path2.starts_with(&path1));
        assert_ne!(path1.starts_with(&path2), true);
    }

    #[test]
    fn not_starts_with_path_1() {
        let segm11 = SynSegment::new("rule-name9".to_string(), "some text1".to_string(), false);
        let segm12 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), true);
        let segm13 = SynSegment::new("rule-name3".to_string(), "some text3".to_string(), true);
        let segms1 = vec![&segm11, &segm12, &segm13];
        let path1 = SynPath::new(segms1);
        let segm21 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
        let segm22 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), false);
        let segm23 = SynSegment::new("rule-name3".to_string(), "some text3".to_string(), false);
        let segm24 = SynSegment::new("rule-name4".to_string(), "some text4".to_string(), false);
        let segm25 = SynSegment::new("rule-name5".to_string(), "some text5".to_string(), true);
        let segms2 = vec![&segm21, &segm22, &segm23, &segm24, &segm25];
        let path2 = SynPath::new(segms2);
        assert_ne!(path2.starts_with(&path1), true);
        assert_ne!(path1.starts_with(&path2), true);
    }

    #[test]
    fn not_starts_with_path_2() {
        let segm11 = SynSegment::new("rule-name1".to_string(), "some text9".to_string(), false);
        let segm12 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), true);
        let segm13 = SynSegment::new("rule-name3".to_string(), "some text3".to_string(), true);
        let segms1 = vec![&segm11, &segm12, &segm13];
        let path1 = SynPath::new(segms1);
        let segm21 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
        let segm22 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), false);
        let segm23 = SynSegment::new("rule-name3".to_string(), "some text3".to_string(), false);
        let segm24 = SynSegment::new("rule-name4".to_string(), "some text4".to_string(), false);
        let segm25 = SynSegment::new("rule-name5".to_string(), "some text5".to_string(), true);
        let segms2 = vec![&segm21, &segm22, &segm23, &segm24, &segm25];
        let path2 = SynPath::new(segms2);
        assert_ne!(path2.starts_with(&path1), true);
        assert_ne!(path1.starts_with(&path2), true);
    }

//    #[test]
//    fn paths_after_1() {
//        let segm11 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
//        let segms1 = vec![&segm11];
//        let path1 = SynPath::new(segms1);
//        let segm21 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
//        let segm22 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), false);
//        let segms2 = vec![&segm21, &segm22];
//        let path2 = SynPath::new(segms2);
//        let segm31 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
//        let segm32 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), false);
//        let segm33 = SynSegment::new("rule-name3".to_string(), "some text3".to_string(), false);
//        let segms3 = vec![&segm31, &segm32, &segm33];
//        let path3 = SynPath::new(segms3);
//        let segm41 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
//        let segm43 = SynSegment::new("rule-name3".to_string(), "some text3".to_string(), false);
//        let segm44 = SynSegment::new("rule-name4".to_string(), "some text4".to_string(), false);
//        let segms4 = vec![&segm41, &segm43, &segm44];
//        let path4 = SynPath::new(segms4);
//        let segm51 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
//        let segm53 = SynSegment::new("rule-name3".to_string(), "some text3".to_string(), false);
//        let segm54 = SynSegment::new("rule-name4".to_string(), "some text4".to_string(), false);
//        let segm55 = SynSegment::new("rule-name5".to_string(), "some text5".to_string(), true);
//        let segms5 = vec![&segm51, &segm53, &segm54, &segm55];
//        let path5 = SynPath::new(segms5);
//
//        let paths = vec![&path1, &path2, &path3, &path4, &path5];
//
//        let segm61 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
//        let segm62 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), false);
//        let segms6 = vec![segm61, segm62];
//        let path6 = SynPath::new(segms6);
//
//        let paths_after1 = path6.paths_after(paths.as_slice(), false);
//        assert_eq!(paths_after1.len(), 5);
//
//        let paths2 = vec![&path2, &path3, &path4, &path5];
//
//        let paths_after2 = path6.paths_after(&paths2, false);
//        assert_eq!(paths_after2.len(), 4);
//
//        let paths_after3 = path6.paths_after(&paths, true);
//        assert_eq!(paths_after3.len(), 2);
//
//        let segm71 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
//        let segms7 = vec![&segm71];
//        let path7 = SynPath::new(segms7);
//
//        let paths_after4 = path7.paths_after(&paths2, false);
//        assert_eq!(paths_after4.len(), 1);
//
//        let paths_after5 = path7.paths_after(&paths2, true);
//        assert_eq!(paths_after5.len(), 1);
//    }

    #[test]
    fn var_range_1() {
        let segm11 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
        let segm12 = SynSegment::new("v_rule-name2".to_string(), "some text2".to_string(), true);
        let segms1 = vec![&segm11, &segm12];
        let path1 = SynPath::new(segms1);
        assert!(path1.value.in_var_range);
    }

    #[test]
    fn not_var_range_1() {
        let segm11 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
        let segm12 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), true);
        let segms1 = vec![&segm11, &segm12];
        let path1 = SynPath::new(segms1);
        assert!(!path1.value.in_var_range);
    }
    
    #[test]
    fn substitute_1() {
        let segm11 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
        let segm12 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), false);
        let segm13 = SynSegment::new("rule-name3".to_string(), "some text3".to_string(), true);
        let segms1 = vec![&segm11, &segm12, &segm13];
        let path1 = SynPath::new(segms1);
        let segm23 = SynSegment::new("rule-name3".to_string(), "some text3".to_string(), false);
        let segm24 = SynSegment::new("rule-name4".to_string(), "some text4".to_string(), false);
        let mut matching: SynMatching = HashMap::new();
        matching.insert(&segm23, &segm24);
        let (new_path, old_path) = path1.substitute(&matching);

        assert_eq!(new_path.value.name, "rule-name4");
        assert_eq!(old_path.unwrap().value.name, "rule-name3");
    }

    #[test]
    fn substitute_2() {
        let segm11 = SynSegment::new("rule-name1".to_string(), "some text1".to_string(), false);
        let segm12 = SynSegment::new("rule-name2".to_string(), "some text2".to_string(), false);
        let segm13 = SynSegment::new("rule-name3".to_string(), "some text3".to_string(), true);
        let segms1 = vec![&segm11, &segm12, &segm13];
        let path1 = SynPath::new(segms1);
        let segm23 = SynSegment::new("rule-name5".to_string(), "some text3".to_string(), false);
        let segm24 = SynSegment::new("rule-name4".to_string(), "some text4".to_string(), false);
        let mut matching: SynMatching = HashMap::new();
        matching.insert(&segm23, &segm24);
        let (new_path, old_path) = path1.substitute(&matching);

        assert_eq!(new_path.value.name, "rule-name3");
        assert_eq!(old_path, None);
    }
}