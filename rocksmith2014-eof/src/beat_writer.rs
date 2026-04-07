use crate::types::{EofEvent, TimeSignature};
use crate::write_utils::*;
use rocksmith2014_xml::InstrumentalArrangement;
use std::collections::HashSet;
use std::io::{self, Write};

pub fn write_beats(
    writer: &mut impl Write,
    inst: &InstrumentalArrangement,
    events: &[EofEvent],
    time_signatures: &[(i32, TimeSignature)],
) -> io::Result<()> {
    let event_beats: HashSet<i32> = events.iter().map(|e| e.beat_number).collect();

    use std::collections::HashMap;
    let ts_map: HashMap<i32, &TimeSignature> =
        time_signatures.iter().map(|(t, ts)| (*t, ts)).collect();

    write_i32_le(writer, inst.ebeats.len() as i32)?;

    let mut prev_tempo: i32 = -1;
    let mut current_den: f64 = 4.0;

    for (index, beat) in inst.ebeats.iter().enumerate() {
        let next_beat = inst.ebeats.get(index + 1);
        let next_beat_time = next_beat.map(|b| b.time).unwrap_or(inst.meta.song_length);

        let event_flag = if event_beats.contains(&(index as i32)) {
            2u32
        } else {
            0u32
        };

        let (ts_flag, den) = if next_beat.is_some() {
            match ts_map.get(&beat.time) {
                Some(ts) => {
                    let (flag, d) = ts_to_flag(ts);
                    current_den = d;
                    (flag, d)
                }
                None => (0u32, current_den),
            }
        } else {
            (0u32, current_den)
        };

        let tempo = if next_beat.is_none() {
            prev_tempo
        } else {
            get_tempo(den, next_beat_time, beat.time)
        };

        let anchor_flag = if prev_tempo == tempo { 0u32 } else { 1u32 };
        prev_tempo = tempo;

        write_i32_le(writer, tempo)?;
        write_i32_le(writer, beat.time)?;
        write_u32_le(writer, anchor_flag | event_flag | ts_flag)?;
        write_i8(writer, 0i8)?;
    }

    Ok(())
}

fn ts_to_flag(ts: &TimeSignature) -> (u32, f64) {
    match ts {
        TimeSignature::TS2_4 => (512, 4.0),
        TimeSignature::TS3_4 => (8, 4.0),
        TimeSignature::TS4_4 => (4, 4.0),
        TimeSignature::TS5_4 => (16, 4.0),
        TimeSignature::TS6_4 => (32, 4.0),
        TimeSignature::Custom(n, d) => {
            let flag = 64u32 | ((*n - 1) << 24) | ((*d - 1) << 16);
            (flag, *d as f64)
        }
    }
}

fn get_tempo(den: f64, next_beat_time: i32, beat_time: i32) -> i32 {
    let beat_length = (next_beat_time - beat_time) as f64 * 1000.0;
    (beat_length * (den / 4.0) + 0.5) as i32
}
