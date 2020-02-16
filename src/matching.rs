use std::collections::HashMap;

use crate::segment::SynSegment;

pub type SynMatching<'a> = HashMap<&'a SynSegment, &'a SynSegment>;

pub fn invert(matching: SynMatching) -> SynMatching {
    let mut inverted: SynMatching = HashMap::new();
    for (key, value) in matching {
        inverted.insert(value, key);
    }
    inverted
}


pub fn get_or_key<'a>(matching: &SynMatching, key: &'a SynSegment) -> SynSegment {
    match matching.get(key) {
        Some(matched) => {
            SynSegment::new(&matched.name, &matched.text, true) // XXX true?
            },
        None => {
            key.clone()
        }
    }
}


/**
* Substitute normalized variables in the keys of matching
* with the original variables in varmap,
* which are keyed by the normalized variables
 */
pub fn get_real_matching<'a>(matching: &'a SynMatching, varmap: &'a SynMatching) -> SynMatching<'a> {
    let mut real_matching: SynMatching = HashMap::new();
    for (key, value) in matching {
        let mut new_key = key;
        let maybe_key = varmap.get(key);
        if maybe_key.is_some() {
            new_key = maybe_key.expect("some key");
        }
        real_matching.insert(new_key, value);
    }
    real_matching
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_invert_1() {
        let segm1 = SynSegment::new("rule-name1", "(text )", false);
        let segm2 = SynSegment::new("rule-name3", "text", true);

        let mut matching: SynMatching = HashMap::new();
        matching.insert(&segm1, &segm2);
        let inverted = invert(matching);

        assert_eq!(*inverted.get(&segm2).expect("segment"), &segm1);
    }

    #[test]
    fn get_real_matching_1() {
        let segm11 = SynSegment::new("rule-name1", "(text )", false);
        let segm12 = SynSegment::new("rule-name2", "text", true);
        let mut matching: SynMatching = HashMap::new();
        matching.insert(&segm11, &segm12);

        let segm21 = SynSegment::new("rule-name1", "(text )", true);
        let segm22 = SynSegment::new("rule-name3", "text", true);
        let mut varmap: SynMatching = HashMap::new();
        varmap.insert(&segm21, &segm22);

        let new_matching = get_real_matching(&matching, &varmap);

        assert_eq!(*new_matching.get(&segm22).expect("segment"), &segm12);
    }

    #[test]
    fn matching_getorkey_1() {
        let segm1 = SynSegment::new("rule-name1", "(text )", false);
        let segm2 = SynSegment::new("rule-name3", "text", true);

        let mut matching: SynMatching = HashMap::new();
        matching.insert(&segm1, &segm2);

        assert_eq!(get_or_key(&matching, &segm2), segm2);
        assert_eq!(get_or_key(&matching, &segm1), segm2);
    }
}
