use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::constants;
use crate::segment::Segment;

#[derive(Debug, Clone)]
pub struct Path<'a> {
    pub value: &'a Segment,
    pub segments: &'a Vec<Segment>,
    identity: Vec<&'a String>,
}

impl<'a> Path<'a> {
    pub fn make_path<'b>(segments: &'a Vec<Segment>) -> Path<'a> {
        let mut identity = Vec::new();
        let l = segments.len();
        for segment in segments.iter() {
            identity.push(&segment.name);
        }
        identity.push(&segments[l - 1].text);
        let value = &segments[l - 1];
        Path {
            value: value,
            segments: segments,
            identity: identity,
        }
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
    pub fn starts_with(&self, path: &Path) -> bool {
        let lself = self.len();
        let lpath = path.len();
        if lself >= lpath && &self.segments[0..lpath] == &path.segments[0..lpath] {
            true
        } else {
            false
        }
    }
    pub fn paths_after(&self, paths: &'a Vec<Path>, try_to_see: bool) -> Vec<&'a Path> {
        let mut seen = false;
        let mut new_paths = Vec::new();
        for path in paths {
            if try_to_see && !seen && path.starts_with(&self) {
                seen = true;
            } else {
                if (!try_to_see || seen) && (!path.starts_with(&self) || path.len() == self.len()) {
                    new_paths.push(path);
                }
            }
        }
        new_paths
    }
}

impl<'a> PartialEq for Path<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl<'a> Eq for Path<'a> {}

impl<'a> Hash for Path<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn make_path_1() {
        let name = "rule-name".to_string();
        let text = "some text".to_string();
        let segm = Segment::make_segment(&name, &text, 0, true);
        let segms = vec![segm];
        let path = Path::make_path(&segms);
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
        let segm = Segment::make_var(0);
        let segms = vec![segm];
        let path = Path::make_path(&segms);
        assert_eq!(path.identity[0], constants::VAR_RULE_NAME);
        assert_eq!(path.identity[1], "<__X0>");
        assert_eq!(path.len(), 1);
        assert_eq!(path.is_var(), true);
        assert_eq!(path.is_leaf(), true);
    }

    #[test]
    fn make_path_2() {
        let segm1 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm2 = Segment::make_segment("rule-name2", "some text2", 0, true);
        let segms = vec![segm1, segm2];
        let path = Path::make_path(&segms);
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
        let segm11 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm12 = Segment::make_segment("rule-name2", "some text2", 0, true);
        let segms1 = vec![segm11, segm12];
        let path1 = Path::make_path(&segms1);
        let segm21 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm22 = Segment::make_segment("rule-name2", "some text2", 0, true);
        let segms2 = vec![segm21, segm22];
        let path2 = Path::make_path(&segms2);
        assert_eq!(&path1, &path2);
        assert_eq!(calculate_hash(&path1), calculate_hash(&path2));
    }

    #[test]
    fn make_paths_2() {
        let segm11 = Segment::make_segment("rule-name1", "some text3", 0, false);
        let segm12 = Segment::make_segment("rule-name2", "some text2", 0, true);
        let segms1 = vec![segm11, segm12];
        let path1 = Path::make_path(&segms1);
        let segm21 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm22 = Segment::make_segment("rule-name2", "some text2", 0, true);
        let segms2 = vec![segm21, segm22];
        let path2 = Path::make_path(&segms2);
        assert_eq!(&path1, &path2);
        assert_eq!(calculate_hash(&path1), calculate_hash(&path2));
    }

    #[test]
    fn starts_with_path_1() {
        let segm11 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm12 = Segment::make_segment("rule-name2", "some text2", 0, true);
        let segms1 = vec![segm11, segm12];
        let path1 = Path::make_path(&segms1);
        let segm21 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm22 = Segment::make_segment("rule-name2", "some text2", 0, true);
        let segms2 = vec![segm21, segm22];
        let path2 = Path::make_path(&segms2);
        assert!(path1.starts_with(&path2));
    }

    #[test]
    fn starts_with_path_2() {
        let segm11 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm12 = Segment::make_segment("rule-name2", "some text2", 0, true);
        let segms1 = vec![segm11, segm12];
        let path1 = Path::make_path(&segms1);
        let segm21 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm22 = Segment::make_segment("rule-name2", "some text2", 0, false);
        let segm23 = Segment::make_segment("rule-name3", "some text3", 0, true);
        let segms2 = vec![segm21, segm22, segm23];
        let path2 = Path::make_path(&segms2);
        assert!(path2.starts_with(&path1));
        assert_ne!(path1.starts_with(&path2), true);
    }

    #[test]
    fn starts_with_path_3() {
        let segm11 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm12 = Segment::make_segment("rule-name2", "some text2", 0, false);
        let segm13 = Segment::make_segment("rule-name3", "some text3", 0, true);
        let segms1 = vec![segm11, segm12, segm13];
        let path1 = Path::make_path(&segms1);
        let segm21 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm22 = Segment::make_segment("rule-name2", "some text2", 0, false);
        let segm23 = Segment::make_segment("rule-name3", "some text3", 0, false);
        let segm24 = Segment::make_segment("rule-name4", "some text4", 0, false);
        let segm25 = Segment::make_segment("rule-name5", "some text5", 0, true);
        let segms2 = vec![segm21, segm22, segm23, segm24, segm25];
        let path2 = Path::make_path(&segms2);
        assert!(path2.starts_with(&path1));
        assert_ne!(path1.starts_with(&path2), true);
    }

    #[test]
    fn not_starts_with_path_1() {
        let segm11 = Segment::make_segment("rule-name9", "some text1", 0, false);
        let segm12 = Segment::make_segment("rule-name2", "some text2", 0, false);
        let segm13 = Segment::make_segment("rule-name3", "some text3", 0, true);
        let segms1 = vec![segm11, segm12, segm13];
        let path1 = Path::make_path(&segms1);
        let segm21 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm22 = Segment::make_segment("rule-name2", "some text2", 0, false);
        let segm23 = Segment::make_segment("rule-name3", "some text3", 0, false);
        let segm24 = Segment::make_segment("rule-name4", "some text4", 0, false);
        let segm25 = Segment::make_segment("rule-name5", "some text5", 0, true);
        let segms2 = vec![segm21, segm22, segm23, segm24, segm25];
        let path2 = Path::make_path(&segms2);
        assert_ne!(path2.starts_with(&path1), true);
        assert_ne!(path1.starts_with(&path2), true);
    }

    #[test]
    fn not_starts_with_path_2() {
        let segm11 = Segment::make_segment("rule-name1", "some text9", 0, false);
        let segm12 = Segment::make_segment("rule-name2", "some text2", 0, false);
        let segm13 = Segment::make_segment("rule-name3", "some text3", 0, true);
        let segms1 = vec![segm11, segm12, segm13];
        let path1 = Path::make_path(&segms1);
        let segm21 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm22 = Segment::make_segment("rule-name2", "some text2", 0, false);
        let segm23 = Segment::make_segment("rule-name3", "some text3", 0, false);
        let segm24 = Segment::make_segment("rule-name4", "some text4", 0, false);
        let segm25 = Segment::make_segment("rule-name5", "some text5", 0, true);
        let segms2 = vec![segm21, segm22, segm23, segm24, segm25];
        let path2 = Path::make_path(&segms2);
        assert_ne!(path2.starts_with(&path1), true);
        assert_ne!(path1.starts_with(&path2), true);
    }

    #[test]
    fn paths_after_1() {
        let segm11 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segms1 = vec![segm11];
        let path1 = Path::make_path(&segms1);
        let segm21 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm22 = Segment::make_segment("rule-name2", "some text2", 0, false);
        let segms2 = vec![segm21, segm22];
        let path2 = Path::make_path(&segms2);
        let segm31 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm32 = Segment::make_segment("rule-name2", "some text2", 0, false);
        let segm33 = Segment::make_segment("rule-name3", "some text3", 0, true);
        let segms3 = vec![segm31, segm32, segm33];
        let path3 = Path::make_path(&segms3);
        let segm41 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm42 = Segment::make_segment("rule-name2", "some text2", 0, false);
        let segm43 = Segment::make_segment("rule-name3", "some text3", 0, false);
        let segm44 = Segment::make_segment("rule-name4", "some text4", 0, false);
        let segms4 = vec![segm41, segm42, segm43, segm44];
        let path4 = Path::make_path(&segms4);
        let segm51 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm52 = Segment::make_segment("rule-name2", "some text2", 0, false);
        let segm53 = Segment::make_segment("rule-name3", "some text3", 0, false);
        let segm54 = Segment::make_segment("rule-name4", "some text4", 0, false);
        let segm55 = Segment::make_segment("rule-name5", "some text5", 0, true);
        let segms5 = vec![segm51, segm52, segm53, segm54, segm55];
        let path5 = Path::make_path(&segms5);

        let paths = vec![path1, path2, path3, path4, path5];

        let segm61 = Segment::make_segment("rule-name1", "some text1", 0, false);
        let segm62 = Segment::make_segment("rule-name2", "some text2", 0, false);
        let segms6 = vec![segm61, segm62];
        let path6 = Path::make_path(&segms6);

        let paths_after = path6.paths_after(&paths, false);
        assert_eq!(paths_after.len(), 2);
    }
}
