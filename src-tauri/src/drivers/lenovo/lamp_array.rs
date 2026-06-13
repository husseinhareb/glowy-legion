//! HID LampArray (usage page 0x59) report handling.
//!
//! Attribute reads use GET_FEATURE and never change lighting state. Updates
//! use the standard SET_FEATURE reports declared by the interface descriptor.

use crate::{
    app::error::AppError,
    domain::{DeviceCapabilities, KeyboardState, LampArrayAttributesSummary, RgbColor},
    drivers::lenovo::protocol::build_zone_rgb_bytes,
    infrastructure::linux::hidraw::LampArrayReportIds,
};

/// Byte length of LampArrayAttributesReport including the report ID prefix:
/// 1 (report ID) + 2 (LampCount) + 4*3 (bounding box) + 4 (kind) + 4 (interval).
const ATTRIBUTES_REPORT_LEN: usize = 23;
const LAMP_UPDATE_FLAG_UPDATE_COMPLETE: u8 = 0x01;
const LAMP_ARRAY_ZONE_COUNT: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LampArrayUpdateReports {
    pub control: Vec<u8>,
    /// One LampRangeUpdateReport per zone, in send order. All but the last
    /// carry LampUpdateComplete=0; the last latches the whole batch.
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
/// left-to-right row (lamp 0 is leftmost). The four logical zones map onto these
/// lamps as contiguous, equal-width blocks, with the rightmost zone absorbing
/// any remainder when `lamp_count` is not divisible by four. Each zone is sent
/// as one LampRangeUpdateReport spanning its lamp range; this is what actually
/// lights the whole keyboard (writing only lamp IDs 0..4 lit just the leftmost
/// lamps). The firmware honours LampIdEnd, verified on the real LOQ.
pub fn build_lamp_array_update_reports(
    state: &KeyboardState,
    capabilities: &DeviceCapabilities,
    report_ids: &LampArrayReportIds,
    lamp_count: u16,
) -> Result<LampArrayUpdateReports, AppError> {
    if !capabilities.supports_zones || capabilities.zone_count as usize != LAMP_ARRAY_ZONE_COUNT {
        return Err(AppError::UnsupportedDevice(
            "HID LampArray backend currently supports only 4-zone keyboards".to_string(),
        ));
    }
    if (lamp_count as usize) < LAMP_ARRAY_ZONE_COUNT {
        return Err(AppError::UnsupportedDevice(format!(
            "LampArray reports {lamp_count} lamps, fewer than the {LAMP_ARRAY_ZONE_COUNT} zones"
        )));
    }

    let control_report_id = report_ids.control.ok_or_else(|| {
        AppError::UnsupportedDevice(
            "the LampArray interface does not declare a control report".to_string(),
        )
    })?;
    let range_update_id = report_ids.range_update.ok_or_else(|| {
        AppError::UnsupportedDevice(
            "the LampArray interface does not declare a range-update report".to_string(),
        )
    })?;

    let colors = zone_colors_for_state(state);
    let intensity = intensity_for_state(state);

    let lamps_per_zone = lamp_count as usize / LAMP_ARRAY_ZONE_COUNT;
    let mut updates = Vec::with_capacity(LAMP_ARRAY_ZONE_COUNT);
    let mut start = 0usize;
    for zone in 0..LAMP_ARRAY_ZONE_COUNT {
        let is_last = zone == LAMP_ARRAY_ZONE_COUNT - 1;
        let end = if is_last {
            lamp_count as usize - 1
        } else {
            start + lamps_per_zone - 1
        };
        updates.push(build_range_update_report(
            range_update_id,
            start as u16,
            end as u16,
            colors[zone],
            intensity,
            is_last,
        ));
        start = end + 1;
    }

    Ok(LampArrayUpdateReports {
        control: build_control_report(control_report_id, false),
        updates,
    })
}

fn build_control_report(report_id: u8, autonomous_mode: bool) -> Vec<u8> {
    vec![report_id, u8::from(autonomous_mode)]
}

/// LampRangeUpdateReport (usage 0x60): report id, LampUpdateFlags(u8),
/// LampIdStart(u16 LE), LampIdEnd(u16 LE), then one RGBI tuple applied to every
/// lamp in `[start, end]`.
fn build_range_update_report(
    report_id: u8,
    start: u16,
    end: u16,
    color: RgbColor,
    intensity: u8,
    update_complete: bool,
) -> Vec<u8> {
    let flags = if update_complete {
        LAMP_UPDATE_FLAG_UPDATE_COMPLETE
    } else {
        0
    };
    let mut report = Vec::with_capacity(10);
    report.push(report_id);
    report.push(flags);
    report.extend_from_slice(&start.to_le_bytes());
    report.extend_from_slice(&end.to_le_bytes());
    report.push(color.r);
    report.push(color.g);
    report.push(color.b);
    report.push(intensity);
    report
}

fn zone_colors_for_state(state: &KeyboardState) -> [RgbColor; LAMP_ARRAY_ZONE_COUNT] {
    let bytes = build_zone_rgb_bytes(state);
    [
        RgbColor::new(bytes[0], bytes[1], bytes[2]),
        RgbColor::new(bytes[3], bytes[4], bytes[5]),
        RgbColor::new(bytes[6], bytes[7], bytes[8]),
        RgbColor::new(bytes[9], bytes[10], bytes[11]),
    ]
}

fn intensity_for_state(state: &KeyboardState) -> u8 {
    if !state.enabled || state.brightness == 0 {
        0
    } else {
        ((state.brightness as u16 * u8::MAX as u16) / 100) as u8
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
    use super::{build_lamp_array_update_reports, parse_lamp_array_attributes_report};
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
            range_update: Some(0x05),
            control: Some(0x06),
            ..LampArrayReportIds::default()
        }
    }

    #[test]
    fn builds_range_updates_for_uniform_static_color() {
        let mut state = KeyboardState::default_static();
        state.secondary_color = None;
        state.primary_color = RgbColor::new(10, 20, 30);
        state.brightness = 50; // intensity = 50 * 255 / 100 = 127

        let reports = build_lamp_array_update_reports(
            &state,
            &DeviceCapabilities::lenovo_lamp_array_4_zone_rgb(),
            &report_ids(),
            24,
        )
        .expect("reports");

        assert_eq!(reports.control, vec![0x06, 0x00]);
        // 24 lamps / 4 zones = 6 lamps per zone, contiguous left-to-right.
        // Each report: id, flags, startLo, startHi, endLo, endHi, R, G, B, I.
        assert_eq!(
            reports.updates,
            vec![
                vec![0x05, 0x00, 0, 0, 5, 0, 10, 20, 30, 127],
                vec![0x05, 0x00, 6, 0, 11, 0, 10, 20, 30, 127],
                vec![0x05, 0x00, 12, 0, 17, 0, 10, 20, 30, 127],
                vec![0x05, 0x01, 18, 0, 23, 0, 10, 20, 30, 127],
            ]
        );
    }

    #[test]
    fn builds_range_updates_for_distinct_zone_colors() {
        let mut state = KeyboardState::default_static();
        state.secondary_color = None;
        state.brightness = 100; // intensity = 255
        state.zone_colors = Some(vec![
            ZoneColor::new(0, RgbColor::new(1, 2, 3)),
            ZoneColor::new(1, RgbColor::new(4, 5, 6)),
            ZoneColor::new(2, RgbColor::new(7, 8, 9)),
            ZoneColor::new(3, RgbColor::new(10, 11, 12)),
        ]);

        let reports = build_lamp_array_update_reports(
            &state,
            &DeviceCapabilities::lenovo_lamp_array_4_zone_rgb(),
            &report_ids(),
            24,
        )
        .expect("reports");

        assert_eq!(
            reports.updates,
            vec![
                vec![0x05, 0x00, 0, 0, 5, 0, 1, 2, 3, 255],
                vec![0x05, 0x00, 6, 0, 11, 0, 4, 5, 6, 255],
                vec![0x05, 0x00, 12, 0, 17, 0, 7, 8, 9, 255],
                vec![0x05, 0x01, 18, 0, 23, 0, 10, 11, 12, 255],
            ]
        );
    }

    #[test]
    fn last_zone_absorbs_remainder_lamps() {
        // 22 lamps / 4 zones = 5 per zone, last zone takes the extra 2.
        let mut state = KeyboardState::default_static();
        state.secondary_color = None;
        state.primary_color = RgbColor::new(1, 1, 1);
        state.brightness = 100;

        let reports = build_lamp_array_update_reports(
            &state,
            &DeviceCapabilities::lenovo_lamp_array_4_zone_rgb(),
            &report_ids(),
            22,
        )
        .expect("reports");

        let ranges: Vec<(u8, u8)> = reports
            .updates
            .iter()
            .map(|r| (r[2], r[4])) // startLo, endLo
            .collect();
        assert_eq!(ranges, vec![(0, 4), (5, 9), (10, 14), (15, 21)]);
        assert_eq!(reports.updates.last().unwrap()[1], 0x01);
    }

    #[test]
    fn off_builds_black_range_updates() {
        let reports = build_lamp_array_update_reports(
            &KeyboardState::off(),
            &DeviceCapabilities::lenovo_lamp_array_4_zone_rgb(),
            &report_ids(),
            24,
        )
        .expect("reports");

        assert_eq!(reports.updates.len(), 4);
        // Off = black at intensity 0 for every zone.
        for update in &reports.updates {
            assert_eq!(&update[6..10], &[0, 0, 0, 0]);
        }
        assert_eq!(reports.updates.last().unwrap()[1], 0x01);
    }

    #[test]
    fn rejects_lamp_count_below_zone_count() {
        let error = build_lamp_array_update_reports(
            &KeyboardState::default_static(),
            &DeviceCapabilities::lenovo_lamp_array_4_zone_rgb(),
            &report_ids(),
            3,
        )
        .expect_err("must reject");
        assert!(matches!(error, AppError::UnsupportedDevice(_)));
    }
}
