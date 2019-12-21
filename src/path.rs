use crate::segment::Segment;

pub struct Path {
    segments: Vec<Segment>,
    identity: Vec<String>,
}

impl Path {
    fn make_path(segments: Vec<Segment>) -> Path {
        let mut identity: Vec<String> = Vec::new();
        let l = segments.len();
        for segment in segments.iter() {
            let name = &segment.name;
            identity.push(name.to_string());
        }
        let text = &segments[l - 1].text;
        identity.push(text.to_string());
        Path {
            segments: segments,
            identity: identity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_path_1() {
        let name = "rule-name".to_string();
        let text = "some text".to_string();
        let segm = Segment::make_segment(&name, &text, 0, true);
        let segms = vec![segm];
        let path = Path::make_path(segms);
        assert_eq!(&path.identity[0], "rule-name");
        assert_eq!(&path.identity[1], "some text");
        assert_eq!(&path.segments[0].name, "rule-name");
        assert_eq!(&path.segments[0].text, "some text");
    }

    #[test]
    fn make_path_2() {
        let name1 = "rule-name1".to_string();
        let text1 = "some text1".to_string();
        let segm1 = Segment::make_segment(&name1, &text1, 0, true);
        let name2 = "rule-name2".to_string();
        let text2 = "some text2".to_string();
        let segm2 = Segment::make_segment(&name2, &text2, 0, true);
        let segms = vec![segm1, segm2];
        let path = Path::make_path(segms);
        assert_eq!(&path.identity[0], "rule-name1");
        assert_eq!(&path.identity[1], "rule-name2");
        assert_eq!(&path.identity[2], "some text2");
        assert_eq!(&path.segments[0].name, "rule-name1");
        assert_eq!(&path.segments[0].text, "some text1");
        assert_eq!(&path.segments[1].name, "rule-name2");
        assert_eq!(&path.segments[1].text, "some text2");
    }
}
