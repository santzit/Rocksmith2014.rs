use rocksmith2014_xml::ShowLight;

use crate::issue::IssueType;

/// Checks show lights for validity.
/// Mirrors ShowLightsChecker.check in the .NET implementation.
pub fn check(lights: &[ShowLight]) -> Option<IssueType> {
    let has_fog = lights
        .iter()
        .any(|sl| sl.note >= ShowLight::FOG_MIN && sl.note <= ShowLight::FOG_MAX);
    let has_beam = lights.iter().any(|sl| {
        (sl.note >= ShowLight::BEAM_MIN && sl.note <= ShowLight::BEAM_MAX)
            || sl.note == ShowLight::BEAM_OFF
    });
    if !has_fog || !has_beam {
        Some(IssueType::InvalidShowlights)
    } else {
        None
    }
}
