//! Tone descriptor inference from tone key names.
//!
//! Mirrors `ToneDescriptors.fs` in Rocksmith2014.Common.Manifest.

/// A single tone descriptor with name, aliases, UI name, and extra flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToneDescriptor {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub ui_name: &'static str,
    pub is_extra: bool,
}

/// All known tone descriptors, in the same order as the .NET reference implementation.
pub static ALL: &[ToneDescriptor] = &[
    ToneDescriptor { name: "Acoustic",       aliases: &["acoustic", "acc", "12str"],          ui_name: "$[35721]ACOUSTIC",      is_extra: false },
    ToneDescriptor { name: "Banjo",          aliases: &["banjo"],                              ui_name: "$[27201]BANJO",         is_extra: true  },
    ToneDescriptor { name: "Bass",           aliases: &["bass"],                               ui_name: "$[35715]BASS",          is_extra: false },
    ToneDescriptor { name: "Chorus",         aliases: &["chorus"],                             ui_name: "$[35723]CHORUS",        is_extra: false },
    ToneDescriptor { name: "Clean",          aliases: &["clean"],                              ui_name: "$[35720]CLEAN",         is_extra: false },
    ToneDescriptor { name: "Crunch",         aliases: &["crunch"],                             ui_name: "$[27156]CRUNCH",        is_extra: true  },
    ToneDescriptor { name: "Delay",          aliases: &["delay"],                              ui_name: "$[35753]DELAY",         is_extra: false },
    ToneDescriptor { name: "Direct",         aliases: &["direct"],                             ui_name: "$[35752]DIRECT",        is_extra: false },
    ToneDescriptor { name: "Distortion",     aliases: &["dist"],                               ui_name: "$[35722]DISTORTION",    is_extra: false },
    ToneDescriptor { name: "Echo",           aliases: &["echo"],                               ui_name: "$[35754]ECHO",          is_extra: false },
    ToneDescriptor { name: "Effect",         aliases: &["effect", "pitch"],                    ui_name: "$[35733]EFFECT",        is_extra: false },
    ToneDescriptor { name: "Emulated",       aliases: &["emu"],                                ui_name: "$[27119]EMULATED",      is_extra: true  },
    ToneDescriptor { name: "Filter",         aliases: &["filter", "wah", "talk"],              ui_name: "$[35729]FILTER",        is_extra: false },
    ToneDescriptor { name: "Flanger",        aliases: &["flange"],                             ui_name: "$[35731]FLANGER",       is_extra: false },
    ToneDescriptor { name: "Fuzz",           aliases: &["fuzz"],                               ui_name: "$[35756]FUZZ",          is_extra: false },
    ToneDescriptor { name: "High Gain",      aliases: &["high", "higain"],                     ui_name: "$[35755]HIGH GAIN",     is_extra: false },
    ToneDescriptor { name: "Lead",           aliases: &["lead", "solo"],                       ui_name: "$[35724]LEAD",          is_extra: false },
    ToneDescriptor { name: "Low Output",     aliases: &["low"],                                ui_name: "$[35732]LOW OUTPUT",    is_extra: false },
    ToneDescriptor { name: "Mandolin",       aliases: &["mandolin"],                           ui_name: "$[27202]MANDOLIN",      is_extra: true  },
    ToneDescriptor { name: "Multi Effect",   aliases: &["multi"],                              ui_name: "$[35751]MULTI-EFFECT",  is_extra: false },
    ToneDescriptor { name: "Octave",         aliases: &["8va", "8vb", "oct"],                  ui_name: "$[35719]OCTAVE",        is_extra: false },
    ToneDescriptor { name: "Overdrive",      aliases: &["od", "drive"],                        ui_name: "$[35716]OVERDRIVE",     is_extra: false },
    ToneDescriptor { name: "Phaser",         aliases: &["phase"],                              ui_name: "$[35730]PHASER",        is_extra: false },
    ToneDescriptor { name: "Piano",          aliases: &["piano"],                              ui_name: "$[29495]PIANO",         is_extra: true  },
    ToneDescriptor { name: "Processed",      aliases: &["synth", "sustain", "processed"],      ui_name: "$[35734]PROCESSED",     is_extra: false },
    ToneDescriptor { name: "Reverb",         aliases: &["verb"],                               ui_name: "$[35726]REVERB",        is_extra: false },
    ToneDescriptor { name: "Rotary",         aliases: &["roto"],                               ui_name: "$[35725]ROTARY",        is_extra: false },
    ToneDescriptor { name: "Special Effect", aliases: &["swell", "organ", "sitar", "sax"],     ui_name: "$[35750]SPECIAL EFFECT",is_extra: false },
    ToneDescriptor { name: "Tremolo",        aliases: &["trem"],                               ui_name: "$[35727]TREMOLO",       is_extra: false },
    ToneDescriptor { name: "Ukulele",        aliases: &["ukulele", "uke"],                     ui_name: "$[27204]UKULELE",       is_extra: true  },
    ToneDescriptor { name: "Vibrato",        aliases: &["vib"],                                ui_name: "$[35728]VIBRATO",       is_extra: false },
    ToneDescriptor { name: "Vocal",          aliases: &["vocal", "vox"],                       ui_name: "$[35718]VOCAL",         is_extra: false },
];

/// Tries to infer tone descriptors from the given tone key name.
///
/// Mirrors `ToneDescriptor.tryInfer` in the .NET reference implementation.
pub fn try_infer(name: &str) -> Vec<&'static ToneDescriptor> {
    let lower = name.to_lowercase();
    ALL.iter()
        .filter(|d| d.aliases.iter().any(|alias| lower.contains(alias)))
        .collect()
}

/// Returns an array of tone descriptors inferred from the tone name,
/// or the Clean tone descriptor as the default (max 3).
///
/// Mirrors `ToneDescriptor.getDescriptionsOrDefault` in the .NET reference.
pub fn get_descriptions_or_default(name: &str) -> Vec<&'static ToneDescriptor> {
    let inferred = try_infer(name);
    if inferred.is_empty() {
        // Fall back to "Clean"
        ALL.iter().filter(|d| d.name == "Clean").collect()
    } else {
        inferred.into_iter().take(3).collect()
    }
}

/// Combines an array of UI names into a space-separated string of description names.
///
/// Mirrors `ToneDescriptor.combineUINames` in the .NET reference implementation.
pub fn combine_ui_names(ui_names: &[&str]) -> String {
    let names: Vec<&str> = ui_names
        .iter()
        .filter_map(|ui| ALL.iter().find(|d| d.ui_name == *ui).map(|d| d.name))
        .collect();
    names.join(" ")
}
