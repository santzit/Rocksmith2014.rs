use rocksmith2014_xml::InstrumentalArrangement;
use crate::types::{EofEvent, EofEventFlag};
use crate::helpers::get_closest_beat;

pub fn create_eof_events(
    get_track_number: &dyn Fn(&InstrumentalArrangement) -> usize,
    beats: &[rocksmith2014_xml::Ebeat],
    inst: &InstrumentalArrangement,
) -> Vec<EofEvent> {
    let track_number = get_track_number(inst) as u16;

    let create = |text: String, time: i32, flag: EofEventFlag| EofEvent {
        text,
        beat_number: get_closest_beat(beats, time) as i32,
        track_number,
        flag,
    };

    let other_events: Vec<EofEvent> = inst.events.iter()
        .filter(|e| !e.code.starts_with("TS"))
        .map(|e| create(e.code.clone(), e.time, EofEventFlag::RS_EVENT))
        .collect();

    let section_events: Vec<EofEvent> = inst.sections.iter()
        .map(|s| create(s.name.clone(), s.start_time, EofEventFlag::RS_SECTION))
        .collect();

    let phrase_events: Vec<EofEvent> = inst.phrase_iterations.iter()
        .map(|p| {
            let phrase = &inst.phrases[p.phrase_id as usize];
            let mut e = create(phrase.name.clone(), p.time, EofEventFlag::RS_PHRASE);
            let is_solo = e.text.to_lowercase().starts_with("solo")
                || section_events.iter().any(|s| s.beat_number == e.beat_number && s.text.to_lowercase().starts_with("solo"));
            if is_solo {
                e.flag |= EofEventFlag::RS_SOLO_PHRASE;
            }
            e
        })
        .collect();

    let mut result = Vec::new();
    result.extend(other_events);
    result.extend(section_events);
    result.extend(phrase_events);
    result
}

pub fn unify_events(track_count: usize, events: Vec<EofEvent>) -> Vec<EofEvent> {
    use std::collections::HashMap;
    let mut groups: HashMap<(String, i32, u16), Vec<usize>> = HashMap::new();
    for (i, e) in events.iter().enumerate() {
        groups.entry((e.text.clone(), e.beat_number, e.flag.bits())).or_default().push(i);
    }

    let mut result: Vec<EofEvent> = events;
    let mut to_remove = vec![false; result.len()];

    for (_, indices) in &groups {
        if indices.len() == track_count {
            if let Some(&first) = indices.first() {
                result[first].track_number = 0;
            }
            for &i in indices.iter().skip(1) {
                to_remove[i] = true;
            }
        }
    }

    let mut i = 0;
    result.retain(|_| { let keep = !to_remove[i]; i += 1; keep });
    result
}
