use rocksmith2014_xml::Anchor;
use rocksmith2014_xml_extension::XmlEntity;

const ERROR_MARGIN: i32 = 3;

fn should_include(entities: &[XmlEntity], start_time: i32, end_time: i32) -> bool {
    entities.iter().any(|e| {
        let time = e.time_code();
        let sustain = e.sustain();
        (time + ERROR_MARGIN >= start_time && time < end_time)
            || (time + sustain + ERROR_MARGIN >= start_time && time + sustain < end_time)
    })
}

pub fn choose(
    entities: &[XmlEntity],
    anchors: &[Anchor],
    phrase_start_time: i32,
    phrase_end_time: i32,
) -> Vec<Anchor> {
    let mut result = Vec::new();

    for (idx, a) in anchors.iter().enumerate() {
        let end_time = anchors
            .get(idx + 1)
            .map(|x| x.time)
            .unwrap_or(phrase_end_time);

        if a.time == phrase_start_time || should_include(entities, a.time, end_time) {
            result.push(a.clone());
        }
    }

    result
}
