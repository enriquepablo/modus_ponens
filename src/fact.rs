use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::segment::SynSegment;
use crate::path::SynPath;
use crate::matching::{ SynMatching, invert };

#[derive(Debug)]
pub struct Fact {
    pub text: String,
    pub paths: Vec<SynPath>,
}

impl Fact {
    fn new(paths: Vec<SynPath>) -> Fact {
        let text = paths.iter().map(|path| path.value.text.clone()).collect::<Vec<String>>().join("");
        Fact { text, paths, }
    }
    pub fn get_all_paths(&self) -> Vec<&SynPath> {
        let mut paths = Vec::new();
        for path in self.paths.iter() {
            if !path.value.text.trim().is_empty() {
                paths.push(path);
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

    pub fn substitute(&self, matching: SynMatching) -> Fact {
        let new_paths = SynPath::substitute_paths(&self.paths, matching);
        // XXX let's check that we do not need to re-parse
        Fact::new(new_paths)
    }
    pub fn normalize (&self) -> (SynMatching, Fact) {
        let mut varmap: SynMatching = HashMap::new();
        let mut counter = 1;
        let leaves = self.get_leaf_paths();
        for path in leaves {
            if path.is_var() {
                let old_var = varmap.get(&path.value);
                if old_var.is_none() {
                    let new_var = SynSegment::make_var(counter);
                    counter += 1;
                    varmap.insert(path.value.clone(), new_var);
                }
            }
        }
        let invarmap = invert(varmap.clone());
        let new_fact = self.substitute(varmap);
        (invarmap, new_fact)
    }
}


impl PartialEq for Fact {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

impl Eq for Fact {}

impl Hash for Fact {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment::SynSegment;

    #[test]
    fn fact_1() {
        let segm11 = SynSegment::new("rule-name1", "(text)", false);
        let segms1 = vec![segm11];
        let path1 = SynPath::new(segms1);

        let segm21 = SynSegment::new("rule-name1", "(text)", false);
        let segm22 = SynSegment::new("rule-name2", "(", true);
        let segms2 = vec![segm21, segm22];
        let path2 = SynPath::new(segms2);

        let segm31 = SynSegment::new("rule-name1", "(text)", false);
        let segm32 = SynSegment::new("rule-name3", "text", true);
        let segms3 = vec![segm31, segm32];
        let path3 = SynPath::new(segms3);

        let segm41 = SynSegment::new("rule-name1", "(text)", false);
        let segm42 = SynSegment::new("rule-name4", ")", true);
        let segms4 = vec![segm41, segm42];
        let path4 = SynPath::new(segms4);

        let paths = vec![path1, path2, path3, path4];
        let fact = Fact::new(paths);

        assert_eq!(fact.get_all_paths().len(), 4);
        assert_eq!(fact.get_leaf_paths().len(), 3);
    }

    #[test]
    fn fact_2() {
        let segm11 = SynSegment::new("rule-name1", "(text )", false);
        let segms1 = vec![segm11];
        let path1 = SynPath::new(segms1);

        let segm21 = SynSegment::new("rule-name1", "(text )", false);
        let segm22 = SynSegment::new("rule-name2", "(", true);
        let segms2 = vec![segm21, segm22];
        let path2 = SynPath::new(segms2);

        let segm31 = SynSegment::new("rule-name1", "(text )", false);
        let segm32 = SynSegment::new("rule-name3", "text", true);
        let segms3 = vec![segm31, segm32];
        let path3 = SynPath::new(segms3);

        let segm41 = SynSegment::new("rule-name1", "(text )", false);
        let segm42 = SynSegment::new("rule-name4", " ", true);
        let segms4 = vec![segm41, segm42];
        let path4 = SynPath::new(segms4);

        let segm51 = SynSegment::new("rule-name1", "(text )", false);
        let segm52 = SynSegment::new("rule-name5", ")", true);
        let segms5 = vec![segm51, segm52];
        let path5 = SynPath::new(segms5);

        let paths = vec![path1, path2, path3, path4, path5];
        let fact = Fact::new(paths);

        assert_eq!(fact.get_all_paths().len(), 4);
        assert_eq!(fact.get_leaf_paths().len(), 3);
    }
}
