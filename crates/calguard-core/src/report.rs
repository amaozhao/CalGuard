use crate::error::CoreError;
use crate::model::{AnalysisReport, Conflict, EventInstance, FreeBlockType, ReportPrivacyOptions};
use serde::Serialize;

pub fn render_markdown_report(report: &AnalysisReport, privacy: &ReportPrivacyOptions) -> String {
    let mut output = String::new();
    output.push_str("# Calendar Health Report\n\n");
    output.push_str(&format!(
        "Period: {} to {}\n",
        report.period_start.format("%Y-%m-%d"),
        report.period_end.format("%Y-%m-%d")
    ));
    output.push_str(&format!(
        "Generated at: {}\n\n",
        report.generated_at.format("%Y-%m-%d %H:%M")
    ));

    output.push_str("## Summary\n\n");
    output.push_str(&format!("Score: {} / 100\n", report.score.score));
    output.push_str(&format!("Grade: {:?}\n", report.score.grade));
    output.push_str(&format!("Conflicts: {}\n", active_conflict_count(report)));
    output.push_str(&format!(
        "Deep Work Hours: {:.1}h\n",
        report.focus_metrics.focus_minutes as f32 / 60.0
    ));
    output.push_str(&format!(
        "Overloaded Days: {}\n\n",
        report.overloaded_days.len()
    ));

    output.push_str("## Top Risks\n\n");
    if report.score.negative_reasons.is_empty() {
        output.push_str("No major risks detected.\n\n");
    } else {
        for (index, reason) in report.score.negative_reasons.iter().take(5).enumerate() {
            output.push_str(&format!(
                "{}. {} ({} pts)\n",
                index + 1,
                reason.label,
                reason.points
            ));
        }
        output.push('\n');
    }

    output.push_str("## Conflicts\n\n");
    let active_conflicts = report
        .conflicts
        .iter()
        .filter(|conflict| !conflict.ignored)
        .collect::<Vec<_>>();
    if active_conflicts.is_empty() {
        output.push_str("No active conflicts.\n\n");
    } else {
        for conflict in active_conflicts {
            output.push_str(&format!(
                "- {}-{} ({:?}, {}m)\n",
                conflict.starts_at.format("%Y-%m-%d %H:%M"),
                conflict.ends_at.format("%H:%M"),
                conflict.severity,
                conflict.overlap_minutes
            ));
            for event in &conflict.events {
                output.push_str(&format!("  - {}\n", display_event_title(event, privacy)));
            }
        }
        output.push('\n');
    }

    output.push_str("## Free Time\n\n");
    for block in report
        .free_blocks
        .iter()
        .filter(|block| block.block_type != FreeBlockType::MicroGap)
    {
        output.push_str(&format!(
            "- {}-{} {:?}\n",
            block.starts_at.format("%A %H:%M"),
            block.ends_at.format("%H:%M"),
            block.block_type
        ));
    }
    output.push('\n');

    output.push_str("## Suggestions\n\n");
    if report.suggestions.is_empty() {
        output.push_str("No suggestions.\n");
    } else {
        for (index, suggestion) in report.suggestions.iter().enumerate() {
            let redacted = RedactedSuggestion::from_suggestion(suggestion, privacy);
            output.push_str(&format!(
                "{}. {}\n   Reason: {}\n",
                index + 1,
                redacted.text,
                redacted.reason
            ));
        }
    }

    output
}

pub fn render_json_report(
    report: &AnalysisReport,
    privacy: &ReportPrivacyOptions,
) -> Result<String, CoreError> {
    let redacted = RedactedReport::from_report(report, privacy);
    serde_json::to_string_pretty(&redacted).map_err(|err| CoreError::Serialization(err.to_string()))
}

fn active_conflict_count(report: &AnalysisReport) -> usize {
    report
        .conflicts
        .iter()
        .filter(|conflict| !conflict.ignored)
        .count()
}

fn display_event_title(event: &EventInstance, privacy: &ReportPrivacyOptions) -> String {
    if privacy.include_event_titles {
        event.title.clone()
    } else {
        "Busy Event".to_string()
    }
}

#[derive(Debug, Serialize)]
struct RedactedReport<'a> {
    schema_version: &'a str,
    period_start: String,
    period_end: String,
    generated_at: String,
    score: &'a crate::model::HealthScore,
    conflicts: Vec<RedactedConflict>,
    free_blocks: &'a [crate::model::FreeBlock],
    overloaded_days: &'a [crate::model::OverloadedDay],
    focus_metrics: &'a crate::model::FocusMetrics,
    suggestions: Vec<RedactedSuggestion>,
}

impl<'a> RedactedReport<'a> {
    fn from_report(report: &'a AnalysisReport, privacy: &ReportPrivacyOptions) -> Self {
        Self {
            schema_version: &report.schema_version,
            period_start: report.period_start.to_rfc3339(),
            period_end: report.period_end.to_rfc3339(),
            generated_at: report.generated_at.to_rfc3339(),
            score: &report.score,
            conflicts: report
                .conflicts
                .iter()
                .map(|conflict| RedactedConflict::from_conflict(conflict, privacy))
                .collect(),
            free_blocks: &report.free_blocks,
            overloaded_days: &report.overloaded_days,
            focus_metrics: &report.focus_metrics,
            suggestions: report
                .suggestions
                .iter()
                .map(|suggestion| RedactedSuggestion::from_suggestion(suggestion, privacy))
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct RedactedConflict {
    id: String,
    event_ids: Vec<String>,
    events: Vec<RedactedEvent>,
    starts_at: String,
    ends_at: String,
    overlap_minutes: i64,
    severity: crate::model::ConflictSeverity,
    reason: String,
    ignored: bool,
}

impl RedactedConflict {
    fn from_conflict(conflict: &Conflict, privacy: &ReportPrivacyOptions) -> Self {
        Self {
            id: conflict.id.clone(),
            event_ids: conflict.event_ids.clone(),
            events: conflict
                .events
                .iter()
                .map(|event| RedactedEvent::from_event(event, privacy))
                .collect(),
            starts_at: conflict.starts_at.to_rfc3339(),
            ends_at: conflict.ends_at.to_rfc3339(),
            overlap_minutes: conflict.overlap_minutes,
            severity: conflict.severity,
            reason: conflict.reason.clone(),
            ignored: conflict.ignored,
        }
    }
}

#[derive(Debug, Serialize)]
struct RedactedEvent {
    id: String,
    source_id: Option<String>,
    title: String,
    location: Option<String>,
    description: Option<String>,
    starts_at: String,
    ends_at: String,
}

impl RedactedEvent {
    fn from_event(event: &EventInstance, privacy: &ReportPrivacyOptions) -> Self {
        Self {
            id: event.id.clone(),
            source_id: privacy
                .include_source_names
                .then(|| event.source_id.clone()),
            title: display_event_title(event, privacy),
            location: privacy
                .include_locations
                .then(|| event.location.clone())
                .flatten(),
            description: privacy
                .include_descriptions
                .then(|| event.description.clone())
                .flatten(),
            starts_at: event.starts_at.to_rfc3339(),
            ends_at: event.ends_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
struct RedactedSuggestion {
    id: String,
    text: String,
    reason: String,
    date: Option<String>,
}

impl RedactedSuggestion {
    fn from_suggestion(
        suggestion: &crate::model::Suggestion,
        privacy: &ReportPrivacyOptions,
    ) -> Self {
        if privacy.include_event_titles {
            return Self {
                id: suggestion.id.clone(),
                text: suggestion.text.clone(),
                reason: suggestion.reason.clone(),
                date: suggestion.date.clone(),
            };
        }

        Self {
            id: suggestion.id.clone(),
            text: match suggestion.kind {
                crate::model::SuggestionKind::MoveEvent => {
                    "Review a conflicting busy event.".to_string()
                }
                crate::model::SuggestionKind::AddFocusBlock => suggestion.text.clone(),
                crate::model::SuggestionKind::AddBuffer => suggestion.text.clone(),
                crate::model::SuggestionKind::DeclineOrShortenMeeting => {
                    "Review a busy event that may need to be shortened.".to_string()
                }
            },
            reason: suggestion.reason.clone(),
            date: suggestion.date.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::analyze_calendar;
    use crate::model::{AnalysisInput, AnalysisSettings, Event, EventStatus, Transparency};
    use chrono::{TimeZone, Utc};

    #[test]
    fn privacy_mode_redacts_titles_locations_and_descriptions() {
        let event = Event {
            id: "e1".to_string(),
            source_id: "s1".to_string(),
            uid: "u1".to_string(),
            title: "Private Title".to_string(),
            description: Some("Secret description".to_string()),
            location: Some("Secret room".to_string()),
            starts_at: Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).single().unwrap(),
            ends_at: Utc.with_ymd_and_hms(2026, 6, 1, 10, 0, 0).single().unwrap(),
            timezone: None,
            is_all_day: false,
            status: EventStatus::Confirmed,
            transparency: Transparency::Opaque,
            recurrence_rule: None,
            excluded_dates: Vec::new(),
            recurrence_id: None,
        };
        let mut other = event.clone();
        other.id = "e2".to_string();
        other.uid = "u2".to_string();
        other.starts_at = Utc.with_ymd_and_hms(2026, 6, 1, 9, 30, 0).single().unwrap();
        other.ends_at = Utc
            .with_ymd_and_hms(2026, 6, 1, 10, 30, 0)
            .single()
            .unwrap();
        let report = analyze_calendar(AnalysisInput {
            events: vec![event, other],
            settings: AnalysisSettings {
                range_days: 1,
                ..Default::default()
            },
            enabled_source_ids: vec!["s1".to_string()],
            ignored_conflict_hashes: Vec::new(),
            generated_at: Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).single().unwrap(),
        });

        let json = render_json_report(
            &report,
            &ReportPrivacyOptions {
                include_event_titles: false,
                include_locations: false,
                include_descriptions: false,
                include_source_names: true,
            },
        )
        .unwrap();

        assert!(json.contains("Busy Event"));
        assert!(!json.contains("Private Title"));
        assert!(!json.contains("Secret room"));
        assert!(!json.contains("Secret description"));

        let markdown = render_markdown_report(
            &report,
            &ReportPrivacyOptions {
                include_event_titles: false,
                include_locations: false,
                include_descriptions: false,
                include_source_names: true,
            },
        );
        assert!(markdown.contains("Busy Event"));
        assert!(!markdown.contains("Private Title"));
    }
}
