use std::hash::{Hash, Hasher};

use crate::path::Path;
use crate::segment::Segment;

#[derive(Debug)]
pub struct Fact<'a> {
    text: String,
    paths: Vec<&'a Path<'a>>,
}

impl<'a> Fact<'a> {
    fn make_fact(text: String, paths: Vec<&'a Path<'a>>) -> Fact<'a> {
        Fact {
            text: text,
            paths: paths,
        }
    }
    fn get_all_paths(&self) -> Vec<&Path> {
        let mut paths = Vec::new();
        for path in self.paths.iter() {
            let l = path.len();
            if !path.segments[l - 1].text.trim().is_empty() {
                paths.push(*path);
            }
        }
        paths
    }
    fn get_leaf_paths(&self) -> Vec<&Path> {
        let mut paths = Vec::new();
        for path in self.paths.iter() {
            let l = path.len();
            if path.is_leaf() && !path.segments[l - 1].text.trim().is_empty() {
                paths.push(*path);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fact_1() {
        let segm11 = Segment::make_segment("rule-name1", "(text)", 0, false);
        let segms1 = vec![segm11];
        let path1 = Path::make_path(&segms1);

        let segm21 = Segment::make_segment("rule-name1", "(text)", 0, false);
        let segm22 = Segment::make_segment("rule-name2", "(", 0, true);
        let segms2 = vec![segm21, segm22];
        let path2 = Path::make_path(&segms2);

        let segm31 = Segment::make_segment("rule-name1", "(text)", 0, false);
        let segm32 = Segment::make_segment("rule-name3", "text", 0, true);
        let segms3 = vec![segm31, segm32];
        let path3 = Path::make_path(&segms3);

        let segm41 = Segment::make_segment("rule-name1", "(text)", 0, false);
        let segm42 = Segment::make_segment("rule-name4", ")", 0, true);
        let segms4 = vec![segm41, segm42];
        let path4 = Path::make_path(&segms4);

        let paths = vec![&path1, &path2, &path3, &path4];
        let fact = Fact::make_fact("(text)".to_string(), paths);

        assert_eq!(fact.get_all_paths().len(), 4);
        assert_eq!(fact.get_leaf_paths().len(), 3);
    }

    #[test]
    fn fact_2() {
        let segm11 = Segment::make_segment("rule-name1", "(text )", 0, false);
        let segms1 = vec![segm11];
        let path1 = Path::make_path(&segms1);

        let segm21 = Segment::make_segment("rule-name1", "(text )", 0, false);
        let segm22 = Segment::make_segment("rule-name2", "(", 0, true);
        let segms2 = vec![segm21, segm22];
        let path2 = Path::make_path(&segms2);

        let segm31 = Segment::make_segment("rule-name1", "(text )", 0, false);
        let segm32 = Segment::make_segment("rule-name3", "text", 0, true);
        let segms3 = vec![segm31, segm32];
        let path3 = Path::make_path(&segms3);

        let segm41 = Segment::make_segment("rule-name1", "(text )", 0, false);
        let segm42 = Segment::make_segment("rule-name4", " ", 0, true);
        let segms4 = vec![segm41, segm42];
        let path4 = Path::make_path(&segms4);

        let segm51 = Segment::make_segment("rule-name1", "(text )", 0, false);
        let segm52 = Segment::make_segment("rule-name5", ")", 0, true);
        let segms5 = vec![segm51, segm52];
        let path5 = Path::make_path(&segms5);

        let paths = vec![&path1, &path2, &path3, &path4, &path5];
        let fact = Fact::make_fact("(text)".to_string(), paths);

        assert_eq!(fact.get_all_paths().len(), 4);
        assert_eq!(fact.get_leaf_paths().len(), 3);
    }
}
