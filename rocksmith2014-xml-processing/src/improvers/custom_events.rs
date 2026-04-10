use rocksmith2014_xml::{ArrangementEvent, InstrumentalArrangement};

/// Processes custom events in the arrangement.
/// Handles "w3", "w3-XX", and "removebeats" events.
/// Mirrors CustomEvents.improve in the .NET implementation.
pub fn improve(arr: &mut InstrumentalArrangement) {
    let events: Vec<ArrangementEvent> = arr.events.clone();

    // Anchor width 3 events
    for ev in events.iter().filter(|e| e.code.starts_with("w3")) {
        let fret_override: Option<i8> = ev.code.split('-').nth(1).and_then(|s| s.parse().ok());
        for level in &mut arr.levels {
            if let Some(anchor) = level.anchors.iter_mut().find(|a| a.time >= ev.time) {
                anchor.width = 3;
                if let Some(fret) = fret_override {
                    anchor.fret = fret;
                }
            }
        }
        arr.events.retain(|e| e as *const _ != ev as *const _);
    }
    arr.events.retain(|e| !e.code.starts_with("w3"));

    // Remove beats event
    if let Some(ev) = events.iter().find(|e| e.code.eq_ignore_ascii_case("removebeats")) {
        arr.ebeats.retain(|b| b.time < ev.time);
        arr.events.retain(|e| !e.code.eq_ignore_ascii_case("removebeats"));
    }
}
