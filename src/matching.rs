use std::collections::HashMap;

use crate::segment::MPSegment;

pub type MPMatching<'a> = HashMap<&'a MPSegment, &'a MPSegment>;

pub fn invert<'a>(matching: &'a MPMatching) -> MPMatching<'a> {
    let mut inverted: MPMatching = HashMap::with_capacity(matching.capacity());
    for (key, value) in matching {
        inverted.insert(value, key);
    }
    inverted
}

pub fn get_or_key<'a>(matching: &'a MPMatching, key: &'a MPSegment) -> &'a MPSegment {
    match matching.get(key) {
        Some(matched) => {
            *matched
            },
        None => {
            key
        }
    }
}


pub fn get_or_key_owning<'a>(matching: MPMatching<'a>, key: &'a MPSegment) -> &'a MPSegment {
    match matching.get(key) {
        Some(matched) => {
            matched
            },
        None => {
            key
        }
    }
}


pub fn get_real_matching_owning<'a>(matching: MPMatching<'a>, varmap: MPMatching<'a>) -> MPMatching<'a> {
    let mut real_matching: MPMatching = HashMap::with_capacity(matching.capacity());
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

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn matching_invert_1() {
//        let segm1 = MPSegment::new("rule-name1".to_string(), "(text )".to_string(), false);
//        let segm2 = MPSegment::new("rule-name3".to_string(), "text".to_string(), true);
//
//        let mut matching: MPMatching = HashMap::new();
//        matching.insert(&segm1, &segm2);
//        let inverted = invert(&matching);
//
//        assert_eq!(*inverted.get(&segm2).expect("segment"), &segm1);
//    }
//
//
//    #[test]
//    fn matching_getorkey_1() {
//        let segm1 = MPSegment::new("rule-name1".to_string(), "(text )".to_string(), false);
//        let segm2 = MPSegment::new("rule-name3".to_string(), "text".to_string(), true);
//
//        let mut matching: MPMatching = HashMap::new();
//        matching.insert(&segm1, &segm2);
//
//        let new_segm = get_or_key(&matching, &segm2);
//        assert_eq!(new_segm, &segm2);
//        let new_segm = get_or_key(&matching, &segm1);
//        assert_eq!(new_segm, &segm2);
//    }
//}
//