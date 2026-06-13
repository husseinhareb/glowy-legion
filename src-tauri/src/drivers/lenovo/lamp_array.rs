//! HID LampArray (usage page 0x59) report handling.
//!
//! Attribute reads use GET_FEATURE and never change lighting state. Updates
//! use the standard SET_FEATURE reports declared by the interface descriptor.

use crate::{
    app::error::AppError,
    domain::{DeviceCapabilities, KeyboardState, LampArrayAttributesSummary, RgbColor},
    infrastructure::linux::hidraw::LampArrayReportIds,
};

/// Byte length of LampArrayAttributesReport including the report ID prefix:
/// 1 (report ID) + 2 (LampCount) + 4*3 (bounding box) + 4 (kind) + 4 (interval).
const ATTRIBUTES_REPORT_LEN: usize = 23;
const LAMP_UPDATE_FLAG_UPDATE_COMPLETE: u8 = 0x01;
/// Lamps addressed by a single LampMultiUpdateReport, fixed by the report
/// descriptor (8 LampId slots + 8 RGBI tuples).
const LAMP_MULTI_UPDATE_LAMP_COUNT: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LampArrayUpdateReports {
    pub control: Vec<u8>,
    /// One or more LampMultiUpdateReports covering every lamp, in send order.
    /// All but the last carry LampUpdateComplete=0; the last latches the batch.
    pub updates: Vec<Vec<u8>>,
}

/// Decode a LampArrayAttributesReport as returned by GET_FEATURE (report ID in
/// byte 0, then little-endian fields per the HID lighting spec).
pub fn parse_lamp_array_attributes_report(
    bytes: &[u8],
) -> Result<LampArrayAttributesSummary, String> {
    if bytes.len() < ATTRIBUTES_REPORT_LEN {
        return Err(format!(
            "LampArrayAttributes report too short: {} bytes (expected at least {})",
            bytes.len(),
            ATTRIBUTES_REPORT_LEN
        ));
    }

    let u16_at = |start: usize| u16::from_le_bytes([bytes[start], bytes[start + 1]]);
    let u32_at = |start: usize| {
        u32::from_le_bytes([
            bytes[start],
            bytes[start + 1],
            bytes[start + 2],
            bytes[start + 3],
        ])
    };

    let lamp_count = u16_at(1);
    let lamp_array_kind = u32_at(15);

    Ok(LampArrayAttributesSummary {
        lamp_count,
        lamp_array_kind,
        kind_label: lamp_array_kind_label(lamp_array_kind).to_string(),
        min_update_interval_microseconds: u32_at(19),
        bounding_box_width_micrometers: u32_at(3),
        bounding_box_height_micrometers: u32_at(7),
        bounding_box_depth_micrometers: u32_at(11),
    })
}

/// Build the LampArray reports for `state`.
///
/// The keyboard exposes `lamp_count` individually addressable lamps in a single
/// left-to-right row (lamp 0 is leftmost), each its own colour segment. The
/// per-lamp framebuffer (see `resolve_lamp_colors`) is sent as a sequence of
/// LampMultiUpdateReports, eight lamps at a time, so every lamp can carry a
/// distinct colour. Only the final report sets LampUpdateComplete, which
/// latches the whole batch at once.
pub fn build_lamp_array_update_reports(
    state: &KeyboardState,
    _capabilities: &DeviceCapabilities,
    report_ids: &LampArrayReportIds,
    lamp_count: u16,
) -> Result<LampArrayUpdateReports, AppError> {
    let lamp_count = lamp_count as usize;
    if lamp_count == 0 {
        return Err(AppError::UnsupportedDevice(
            "the LampArray reports zero lamps".to_string(),
        ));
    }

    let control_report_id = report_ids.control.ok_or_else(|| {
        AppError::UnsupportedDevice(
            "the LampArray interface does not declare a control report".to_string(),
        )
    })?;
    let multi_update_id = report_ids.multi_update.ok_or_else(|| {
        AppError::UnsupportedDevice(
            "the LampArray interface does not declare a multi-update report".to_string(),
        )
    })?;

    let colors = resolve_lamp_colors(state, lamp_count);
    let intensity = intensity_for_state(state);

    let mut updates = Vec::new();
    let mut start = 0usize;
    while start < lamp_count {
        let end = (start + LAMP_MULTI_UPDATE_LAMP_COUNT).min(lamp_count);
        let is_last = end == lamp_count;
        updates.push(build_multi_update_report(
            multi_update_id,
            &colors[start..end],
            start,
            intensity,
            is_last,
        ));
        start = end;
    }

    Ok(LampArrayUpdateReports {
        control: build_control_report(control_report_id, false),
        updates,
    })
}

fn build_control_report(report_id: u8, autonomous_mode: bool) -> Vec<u8> {
    vec![report_id, u8::from(autonomous_mode)]
}

/// LampMultiUpdateReport (usage 0x50): report id, LampCount(u8), flags(u8),
/// then a fixed 8 LampId (u16 LE) slots and 8 interleaved RGBI tuples. Lamps
/// beyond `batch` are zero-padded; `start` is the lamp id of `batch[0]`.
fn build_multi_update_report(
    report_id: u8,
    batch: &[RgbColor],
    start: usize,
    intensity: u8,
    update_complete: bool,
) -> Vec<u8> {
    debug_assert!(batch.len() <= LAMP_MULTI_UPDATE_LAMP_COUNT);
    let flags = if update_complete {
        LAMP_UPDATE_FLAG_UPDATE_COMPLETE
    } else {
        0
    };
    let mut report = Vec::with_capacity(51);
    report.push(report_id);
    report.push(batch.len() as u8);
    report.push(flags);

    for slot in 0..LAMP_MULTI_UPDATE_LAMP_COUNT {
        let lamp_id = if slot < batch.len() {
            (start + slot) as u16
        } else {
            0
        };
        report.extend_from_slice(&lamp_id.to_le_bytes());
    }

    for slot in 0..LAMP_MULTI_UPDATE_LAMP_COUNT {
        match batch.get(slot) {
            Some(color) => {
                report.push(color.r);
                report.push(color.g);
                report.push(color.b);
                report.push(intensity);
            }
            None => report.extend_from_slice(&[0, 0, 0, 0]),
        }
    }

    report
}

/// Resolve a per-lamp colour framebuffer of length `lamp_count`, scaled by
/// brightness.
///
/// Each lamp defaults to the primary colour. For zone-aware effects, any
/// `zone_colors` entry whose `zone_index` is a valid lamp id overrides that
/// lamp — with `zone_count == lamp_count` this is a direct per-lamp palette.
/// Off / disabled / zero-brightness states resolve to all black.
///
/// The LOQ firmware ignores the per-lamp Intensity channel (any non-zero value
/// reads as fully on), verified on real hardware, so brightness is applied by
/// scaling the RGB channels themselves, which the firmware does honour.
fn resolve_lamp_colors(state: &KeyboardState, lamp_count: usize) -> Vec<RgbColor> {
    let black = RgbColor::new(0, 0, 0);
    if state.effect == crate::domain::LightingEffect::Off
        || !state.enabled
        || state.brightness == 0
    {
        return vec![black; lamp_count];
    }

    let mut colors = vec![state.primary_color; lamp_count];

    let zone_aware = matches!(
        state.effect,
        crate::domain::LightingEffect::Static | crate::domain::LightingEffect::Breathing
    );
    if zone_aware {
        if let Some(zones) = &state.zone_colors {
            for zone in zones {
                let lamp = zone.zone_index as usize;
                if lamp < lamp_count {
                    colors[lamp] = zone.color;
                }
            }
        }
    }

    let scale = |value: u8| ((value as u16 * state.brightness as u16) / 100) as u8;
    colors
        .into_iter()
        .map(|c| RgbColor::new(scale(c.r), scale(c.g), scale(c.b)))
        .collect()
}

/// The Intensity byte the firmware ignores: full on when lit, zero when off.
/// Brightness is carried by the scaled RGB channels (see `resolve_lamp_colors`).
fn intensity_for_state(state: &KeyboardState) -> u8 {
    if !state.enabled || state.brightness == 0 {
        0
    } else {
        u8::MAX
    }
}

/// LampArrayKind values from the HID Lighting & Illumination usage table.
fn lamp_array_kind_label(kind: u32) -> &'static str {
    match kind {
        0x01 => "Keyboard",
        0x02 => "Mouse",
        0x03 => "Game controller",
        0x04 => "Peripheral",
        0x05 => "Scene",
        0x06 => "Notification",
        0x07 => "Chassis",
        0x08 => "Wearable",
        0x09 => "Furniture",
        0x0a => "Art",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_lamp_array_update_reports, parse_lamp_array_attributes_report,
        LAMP_MULTI_UPDATE_LAMP_COUNT,
    };
    use crate::{
        app::error::AppError,
        domain::{DeviceCapabilities, KeyboardState, RgbColor, ZoneColor},
        infrastructure::linux::hidraw::LampArrayReportIds,
    };

    fn attributes_report(report_id: u8, lamp_count: u16, kind: u32, min_interval: u32) -> Vec<u8> {
        let mut bytes = vec![report_id];
        bytes.extend_from_slice(&lamp_count.to_le_bytes());
        bytes.extend_from_slice(&350_000u32.to_le_bytes()); // width
        bytes.extend_from_slice(&120_000u32.to_le_bytes()); // height
        bytes.extend_from_slice(&10_000u32.to_le_bytes()); // depth
        bytes.extend_from_slice(&kind.to_le_bytes());
        bytes.extend_from_slice(&min_interval.to_le_bytes());
        bytes
    }

    #[test]
    fn parses_keyboard_attributes_report() {
        let bytes = attributes_report(0x01, 4, 0x01, 33_333);

        let summary = parse_lamp_array_attributes_report(&bytes).expect("parse");
        assert_eq!(summary.lamp_count, 4);
        assert_eq!(summary.lamp_array_kind, 0x01);
        assert_eq!(summary.kind_label, "Keyboard");
        assert_eq!(summary.min_update_interval_microseconds, 33_333);
        assert_eq!(summary.bounding_box_width_micrometers, 350_000);
    }

    #[test]
    fn unknown_kind_gets_unknown_label() {
        let bytes = attributes_report(0x01, 1, 0x7f, 0);

        let summary = parse_lamp_array_attributes_report(&bytes).expect("parse");
        assert_eq!(summary.kind_label, "Unknown");
    }

    #[test]
    fn short_report_is_rejected() {
        let error = parse_lamp_array_attributes_report(&[0x01, 0x04]).unwrap_err();
        assert!(error.contains("too short"));
    }

    fn report_ids() -> LampArrayReportIds {
        LampArrayReportIds {
            multi_update: Some(0x04),
            control: Some(0x06),
            ..LampArrayReportIds::default()
        }
    }

    // Colors occupy bytes [19..51] of a multi-update report: 8 interleaved RGBI
    // tuples after the report id, LampCount, flags, and 8 LampId (u16) slots.
    const COLORS_AT: usize = 19;

    #[test]
    fn builds_multi_update_for_uniform_static_color() {
        let mut state = KeyboardState::default_static();
        state.secondary_color = None;
        state.primary_color = RgbColor::new(10, 20, 30);
        // Brightness scales the RGB channels (the firmware ignores intensity):
        // 50% -> 5, 10, 15. The intensity byte is full-on (255) when lit.
        state.brightness = 50;

        let reports = build_lamp_array_update_reports(
            &state,
            &DeviceCapabilities::lenovo_lamp_array_rgb(),
            &report_ids(),
            24,
        )
        .expect("reports");

        assert_eq!(reports.control, vec![0x06, 0x00]);
        // 24 lamps -> 3 multi-update reports of 8 lamps each.
        assert_eq!(reports.updates.len(), 3);
        for update in &reports.updates {
            assert_eq!(update.len(), 51);
            assert_eq!(update[0], 0x04); // report id
            assert_eq!(update[1], 8); // LampCount
        }
        // First two reports do not latch; the last does.
        assert_eq!(reports.updates[0][2], 0x00);
        assert_eq!(reports.updates[2][2], 0x01);
        // Lamp ids of the second report are 8..15 (u16 LE).
        assert_eq!(&reports.updates[1][3..7], &[8, 0, 9, 0]);
        // Every lamp carries the scaled uniform color with full intensity.
        let mut expected = Vec::new();
        for _ in 0..LAMP_MULTI_UPDATE_LAMP_COUNT {
            expected.extend_from_slice(&[5, 10, 15, 255]);
        }
        for update in &reports.updates {
            assert_eq!(&update[COLORS_AT..51], expected.as_slice());
        }
    }

    #[test]
    fn builds_multi_update_for_per_lamp_palette() {
        let mut state = KeyboardState::default_static();
        state.secondary_color = None;
        state.brightness = 100; // no scaling
        state.zone_colors = Some(
            (0u8..8)
                .map(|i| ZoneColor::new(i, RgbColor::new(i * 10, i * 10 + 1, i * 10 + 2)))
                .collect(),
        );

        let reports = build_lamp_array_update_reports(
            &state,
            &DeviceCapabilities::lenovo_lamp_array_rgb(),
            &report_ids(),
            8,
        )
        .expect("reports");

        assert_eq!(reports.updates.len(), 1);
        let update = &reports.updates[0];
        assert_eq!(update[1], 8); // LampCount
        assert_eq!(update[2], 0x01); // single report latches
        let mut expected = Vec::new();
        for i in 0u8..8 {
            expected.extend_from_slice(&[i * 10, i * 10 + 1, i * 10 + 2, 255]);
        }
        assert_eq!(&update[COLORS_AT..51], expected.as_slice());
    }

    #[test]
    fn partial_final_batch_is_zero_padded() {
        // 20 lamps -> batches of 8, 8, 4. The last report addresses 4 lamps and
        // zero-pads the remaining slots.
        let mut state = KeyboardState::default_static();
        state.secondary_color = None;
        state.primary_color = RgbColor::new(1, 1, 1);
        state.brightness = 100;

        let reports = build_lamp_array_update_reports(
            &state,
            &DeviceCapabilities::lenovo_lamp_array_rgb(),
            &report_ids(),
            20,
        )
        .expect("reports");

        assert_eq!(reports.updates.len(), 3);
        let last = reports.updates.last().unwrap();
        assert_eq!(last[1], 4); // LampCount = 4 valid lamps
        assert_eq!(last[2], 0x01); // latches
        assert_eq!(last.len(), 51); // still a fixed-size report
        assert_eq!(&last[3..7], &[16, 0, 17, 0]); // first padded lamp ids
        assert_eq!(&last[11..19], &[0; 8]); // padded lamp id slots are zero
        // Padded color tuples (slots 4..8) are zero.
        assert_eq!(&last[COLORS_AT + 16..51], &[0; 16]);
    }

    #[test]
    fn off_builds_black_multi_update() {
        let reports = build_lamp_array_update_reports(
            &KeyboardState::off(),
            &DeviceCapabilities::lenovo_lamp_array_rgb(),
            &report_ids(),
            24,
        )
        .expect("reports");

        assert_eq!(reports.updates.len(), 3);
        for update in &reports.updates {
            assert_eq!(&update[COLORS_AT..51], &[0u8; 32]); // black, intensity 0
        }
        assert_eq!(reports.updates.last().unwrap()[2], 0x01);
    }

    #[test]
    fn rejects_zero_lamp_count() {
        let error = build_lamp_array_update_reports(
            &KeyboardState::default_static(),
            &DeviceCapabilities::lenovo_lamp_array_rgb(),
            &report_ids(),
            0,
        )
        .expect_err("must reject");
        assert!(matches!(error, AppError::UnsupportedDevice(_)));
    }
}
