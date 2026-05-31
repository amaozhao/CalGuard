use calguard_core::{
    analyze_calendar, expand_events, parse_ics, AnalysisInput, AnalysisSettings, ConflictSeverity,
    FreeBlockType,
};
use chrono::{Datelike, TimeZone, Utc};

const FIXTURES: &[(&str, &str)] = &[
    ("simple", include_str!("../../../fixtures/simple.ics")),
    ("conflicts", include_str!("../../../fixtures/conflicts.ics")),
    (
        "recurring-weekly",
        include_str!("../../../fixtures/recurring-weekly.ics"),
    ),
    (
        "recurring-daily",
        include_str!("../../../fixtures/recurring-daily.ics"),
    ),
    (
        "recurring-exdate",
        include_str!("../../../fixtures/recurring-exdate.ics"),
    ),
    ("all-day", include_str!("../../../fixtures/all-day.ics")),
    (
        "transparent",
        include_str!("../../../fixtures/transparent.ics"),
    ),
    ("cancelled", include_str!("../../../fixtures/cancelled.ics")),
    ("timezone", include_str!("../../../fixtures/timezone.ics")),
    (
        "no-dtend-duration",
        include_str!("../../../fixtures/no-dtend-duration.ics"),
    ),
    ("malformed", include_str!("../../../fixtures/malformed.ics")),
    ("cross-day", include_str!("../../../fixtures/cross-day.ics")),
    (
        "touching-boundary",
        include_str!("../../../fixtures/touching-boundary.ics"),
    ),
    (
        "three-way-conflict",
        include_str!("../../../fixtures/three-way-conflict.ics"),
    ),
    ("tentative", include_str!("../../../fixtures/tentative.ics")),
    (
        "lunch-covered",
        include_str!("../../../fixtures/lunch-covered.ics"),
    ),
    (
        "late-meeting",
        include_str!("../../../fixtures/late-meeting.ics"),
    ),
    (
        "no-focus-day",
        include_str!("../../../fixtures/no-focus-day.ics"),
    ),
    (
        "folded-summary",
        include_str!("../../../fixtures/folded-summary.ics"),
    ),
    (
        "escaped-values",
        include_str!("../../../fixtures/escaped-values.ics"),
    ),
];

#[test]
fn all_required_fixtures_parse_without_whole_file_failure() {
    assert!(FIXTURES.len() >= 20);
    for (name, contents) in FIXTURES {
        let source_id = format!("source-{name}");
        let parsed = parse_ics(contents, &source_id);
        if *name == "malformed" {
            assert_eq!(parsed.events.len(), 1, "malformed keeps good events");
            assert!(!parsed.issues.is_empty(), "malformed reports parse issues");
        } else {
            assert!(
                !parsed.events.is_empty(),
                "{name} should have parsed events"
            );
            assert!(
                parsed.issues.is_empty(),
                "{name} issues: {:?}",
                parsed.issues
            );
        }
    }
}

#[test]
fn fixture_conflicts_generate_expected_conflict() {
    let source_id = "source-conflicts".to_string();
    let parsed = parse_ics(include_str!("../../../fixtures/conflicts.ics"), &source_id);
    let report = analyze_calendar(AnalysisInput {
        events: parsed.events,
        settings: AnalysisSettings {
            range_days: 1,
            ..Default::default()
        },
        enabled_source_ids: vec![source_id],
        ignored_conflict_hashes: Vec::new(),
        generated_at: Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).single().unwrap(),
    });

    assert_eq!(report.conflicts.len(), 1);
    assert_eq!(report.conflicts[0].severity, ConflictSeverity::Medium);
}

#[test]
fn fixture_recurrence_expands_count_until_and_exdate() {
    let source_id = "source-recurring".to_string();
    let mut events = Vec::new();
    for file in [
        include_str!("../../../fixtures/recurring-weekly.ics"),
        include_str!("../../../fixtures/recurring-daily.ics"),
        include_str!("../../../fixtures/recurring-exdate.ics"),
    ] {
        events.extend(parse_ics(file, &source_id).events);
    }
    let instances = expand_events(
        &events,
        Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).single().unwrap(),
        Utc.with_ymd_and_hms(2026, 6, 20, 0, 0, 0).single().unwrap(),
        1000,
    );

    assert!(instances
        .iter()
        .any(|event| event.uid == "weekly-1" && event.starts_at.day() == 15));
    assert_eq!(
        instances
            .iter()
            .filter(|event| event.uid == "daily-1")
            .count(),
        5
    );
    assert_eq!(
        instances
            .iter()
            .filter(|event| event.uid == "exdate-1")
            .count(),
        4
    );
}

#[test]
fn free_time_fixture_detects_no_focus_risk() {
    let source_id = "source-no-focus".to_string();
    let parsed = parse_ics(
        include_str!("../../../fixtures/no-focus-day.ics"),
        &source_id,
    );
    let report = analyze_calendar(AnalysisInput {
        events: parsed.events,
        settings: AnalysisSettings {
            range_days: 1,
            ..Default::default()
        },
        enabled_source_ids: vec![source_id],
        ignored_conflict_hashes: Vec::new(),
        generated_at: Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).single().unwrap(),
    });

    assert!(report
        .focus_metrics
        .no_focus_days
        .contains(&"2026-06-01".to_string()));
    assert!(report
        .free_blocks
        .iter()
        .all(|block| block.block_type != FreeBlockType::DeepWork));
}
