use std::collections::HashMap;

use crate::segment::Segment;

pub type Matching<'a> = HashMap<&'a Segment, &'a Segment>;

pub fn invert(matching: Matching) -> Matching {
    let mut inverted: Matching = HashMap::new();
    for (key, value) in matching {
        inverted.insert(value, key);
    }
    inverted
}

pub fn get_real_matching<'a>(matching: &'a Matching, varmap: &'a Matching) -> Matching<'a> {
    let mut real_matching: Matching = HashMap::new();
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
        let segm1 = Segment::make_segment("rule-name1", "(text )", 0, false);
        let segm2 = Segment::make_segment("rule-name3", "text", 0, true);

        let mut matching: Matching = HashMap::new();
        matching.insert(&segm1, &segm2);
        let inverted = invert(matching);

        assert_eq!(*inverted.get(&segm2).expect("bad"), &segm1);
    }

    #[test]
    fn get_real_matching_1() {
        let segm11 = Segment::make_segment("rule-name1", "(text )", 0, false);
        let segm12 = Segment::make_segment("rule-name2", "text", 0, true);
        let mut matching: Matching = HashMap::new();
        matching.insert(&segm11, &segm12);

        let segm21 = Segment::make_segment("rule-name1", "(text )", 0, true);
        let segm22 = Segment::make_segment("rule-name3", "text", 0, true);
        let mut varmap: Matching = HashMap::new();
        varmap.insert(&segm21, &segm22);

        let new_matching = get_real_matching(&matching, &varmap);

        assert_eq!(*new_matching.get(&segm22).expect("bad"), &segm12);
    }
}
