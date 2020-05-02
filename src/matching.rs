use std::collections::HashMap;

use crate::segment::SynSegment;

pub type SynMatching<'a> = HashMap<&'a SynSegment, &'a SynSegment>;

pub fn invert<'a>(matching: &'a SynMatching) -> SynMatching<'a> {
    let mut inverted: SynMatching = HashMap::with_capacity(matching.capacity());
    for (key, value) in matching {
        inverted.insert(value, key);
    }
    inverted
}

pub fn invert_owning<'a>(matching: SynMatching<'a>) -> SynMatching<'a> {
    let mut inverted: SynMatching = HashMap::with_capacity(matching.capacity());
    for (key, value) in matching {
        inverted.insert(value, key);
    }
    inverted
}


pub fn get_or_key<'a>(matching: &'a SynMatching, key: &'a SynSegment) -> &'a SynSegment {
    match matching.get(key) {
        Some(matched) => {
            *matched
            },
        None => {
            key
        }
    }
}


pub fn get_or_key_owning<'a>(matching: SynMatching<'a>, key: &'a SynSegment) -> &'a SynSegment {
    match matching.get(key) {
        Some(matched) => {
            matched
            },
        None => {
            key
        }
    }
}


pub fn get_real_matching_owning<'a>(matching: SynMatching<'a>, varmap: SynMatching<'a>) -> SynMatching<'a> {
    let mut real_matching: SynMatching = HashMap::with_capacity(matching.capacity());
    for (key, value) in matching {
        let mut new_key = key;
        let maybe_key = varmap.get(key);
        if maybe_key.is_some() {
            new_key = maybe_key.expect("some key");
        }
        real_matching.insert(&new_key, &value);
    }
    real_matching
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_invert_1() {
        let segm1 = SynSegment::new("rule-name1".to_string(), "(text )".to_string(), false);
        let segm2 = SynSegment::new("rule-name3".to_string(), "text".to_string(), true);

        let mut matching: SynMatching = HashMap::new();
        matching.insert(&segm1, &segm2);
        let inverted = invert(&matching);

        assert_eq!(*inverted.get(&segm2).expect("segment"), &segm1);
    }


    #[test]
    fn matching_getorkey_1() {
        let segm1 = SynSegment::new("rule-name1".to_string(), "(text )".to_string(), false);
        let segm2 = SynSegment::new("rule-name3".to_string(), "text".to_string(), true);

        let mut matching: SynMatching = HashMap::new();
        matching.insert(&segm1, &segm2);

        let new_segm = get_or_key(&matching, &segm2);
        assert_eq!(new_segm, &segm2);
        let new_segm = get_or_key(&matching, &segm1);
        assert_eq!(new_segm, &segm2);
    }
}
