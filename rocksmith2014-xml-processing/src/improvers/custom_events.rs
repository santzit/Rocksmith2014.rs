use rocksmith2014_xml::{ArrangementEvent, ChordMask, InstrumentalArrangement};

/// Processes custom events in the arrangement.
/// Handles "w3", "w3-XX", "removebeats", and "so" (slide-out) events.
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
        arr.events.retain(|e| !e.code.starts_with("w3"));
    }
    arr.events.retain(|e| !e.code.starts_with("w3"));

    // Remove beats event
    if let Some(ev) = events
        .iter()
        .find(|e| e.code.eq_ignore_ascii_case("removebeats"))
    {
        arr.ebeats.retain(|b| b.time < ev.time);
        arr.events
            .retain(|e| !e.code.eq_ignore_ascii_case("removebeats"));
    }

    // Slide-out ("so") events: adjust handshape end times and remove the event
    let so_times: Vec<i32> = events
        .iter()
        .filter(|e| e.code == "so")
        .map(|e| e.time)
        .collect();
    for so_time in so_times {
        for level in &mut arr.levels {
            // Find the chord or chord notes at this time and get max sustain
            let max_sustain = level
                .chords
                .iter()
                .filter(|c| c.time == so_time || c.mask.contains(ChordMask::LINK_NEXT))
                .flat_map(|c| c.chord_notes.iter())
                .map(|cn| cn.sustain)
                .max();
            // Also check notes at this time
            let note_max_sustain = level
                .notes
                .iter()
                .filter(|n| n.time == so_time)
                .map(|n| n.sustain)
                .max();
            let effective_sustain = max_sustain.or(note_max_sustain);
            if let Some(sustain) = effective_sustain {
                // Adjust handshape that starts at or before so_time to end at so_time + sustain
                if let Some(hs) = level
                    .hand_shapes
                    .iter_mut()
                    .find(|hs| hs.start_time <= so_time && hs.end_time > so_time)
                {
                    hs.end_time = so_time + sustain;
                }
            }
        }
    }
    arr.events.retain(|e| e.code != "so");
}
