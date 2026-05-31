use crate::model::*;

pub fn score_health(
    conflicts: &[Conflict],
    free_blocks: &[FreeBlock],
    overloaded_days: &[OverloadedDay],
    busy_instances: &[EventInstance],
    settings: &AnalysisSettings,
    focus_metrics: &FocusMetrics,
) -> HealthScore {
    let mut raw_score: i16 = 100;
    let mut positive_reasons = Vec::new();
    let mut negative_reasons = Vec::new();

    for conflict in conflicts.iter().filter(|conflict| !conflict.ignored) {
        let points = match conflict.severity {
            ConflictSeverity::Critical => -15,
            ConflictSeverity::High => -10,
            ConflictSeverity::Medium => -5,
            ConflictSeverity::Low => -2,
        };
        raw_score += points;
        negative_reasons.push(ScoreReason {
            id: format!("score-conflict-{}", conflict.id),
            label: format!(
                "{:?} conflict at {}",
                conflict.severity,
                conflict.starts_at.format("%Y-%m-%d %H:%M")
            ),
            points,
            date: Some(conflict.starts_at.date_naive().to_string()),
            event_ids: conflict.event_ids.clone(),
        });
    }

    for date in &focus_metrics.no_focus_days {
        raw_score -= 8;
        negative_reasons.push(ScoreReason {
            id: format!("score-no-focus-{date}"),
            label: format!("{date} has no deep work block"),
            points: -8,
            date: Some(date.clone()),
            event_ids: Vec::new(),
        });
    }

    for day in overloaded_days {
        if day.meeting_minutes > settings.overload_meeting_minutes {
            raw_score -= 8;
            negative_reasons.push(ScoreReason {
                id: format!("score-overload-{}", day.date),
                label: format!("{} has more than 5h of meetings", day.date),
                points: -8,
                date: Some(day.date.clone()),
                event_ids: Vec::new(),
            });
        }
        if day.meeting_minutes > settings.severe_overload_meeting_minutes {
            raw_score -= 7;
            negative_reasons.push(ScoreReason {
                id: format!("score-severe-overload-{}", day.date),
                label: format!("{} has more than 7h of meetings", day.date),
                points: -7,
                date: Some(day.date.clone()),
                event_ids: Vec::new(),
            });
        }
        if day.reasons.iter().any(|reason| reason.contains("19:00")) {
            raw_score -= 5;
            negative_reasons.push(ScoreReason {
                id: format!("score-late-{}", day.date),
                label: format!("{} has a meeting after 19:00", day.date),
                points: -5,
                date: Some(day.date.clone()),
                event_ids: Vec::new(),
            });
        }
        if day
            .reasons
            .iter()
            .any(|reason| reason.contains("consecutive meetings"))
        {
            raw_score -= 5;
            negative_reasons.push(ScoreReason {
                id: format!("score-consecutive-{}", day.date),
                label: format!("{} has more than 3 consecutive meetings", day.date),
                points: -5,
                date: Some(day.date.clone()),
                event_ids: Vec::new(),
            });
        }
        if day.reasons.iter().any(|reason| reason.contains("lunch")) {
            raw_score -= 5;
            negative_reasons.push(ScoreReason {
                id: format!("score-lunch-{}", day.date),
                label: format!("{} lunch is covered by meetings", day.date),
                points: -5,
                date: Some(day.date.clone()),
                event_ids: Vec::new(),
            });
        }
    }

    let micro_gap_days = free_blocks
        .iter()
        .filter(|block| block.block_type == FreeBlockType::MicroGap)
        .fold(
            std::collections::HashMap::<String, usize>::new(),
            |mut days, block| {
                *days
                    .entry(block.starts_at.date_naive().to_string())
                    .or_default() += 1;
                days
            },
        );
    for (date, count) in micro_gap_days {
        if count > 3 {
            raw_score -= 3;
            negative_reasons.push(ScoreReason {
                id: format!("score-microgap-{date}"),
                label: format!("{date} has more than 3 micro gaps"),
                points: -3,
                date: Some(date),
                event_ids: Vec::new(),
            });
        }
    }

    if conflicts.iter().all(|conflict| conflict.ignored) {
        raw_score += 8;
        positive_reasons.push(ScoreReason {
            id: "score-no-conflicts".to_string(),
            label: "No active conflicts in the analysis period".to_string(),
            points: 8,
            date: None,
            event_ids: Vec::new(),
        });
    }

    if focus_metrics.no_focus_days.is_empty() {
        raw_score += 8;
        positive_reasons.push(ScoreReason {
            id: "score-focus-every-workday".to_string(),
            label: "Every workday has a deep work block".to_string(),
            points: 8,
            date: None,
            event_ids: Vec::new(),
        });
    }

    if has_meeting_free_day(busy_instances) {
        raw_score += 5;
        positive_reasons.push(ScoreReason {
            id: "score-meeting-free-day".to_string(),
            label: "At least one day has no meetings".to_string(),
            points: 5,
            date: None,
            event_ids: Vec::new(),
        });
    }

    if overloaded_days.iter().all(|day| day.meeting_minutes < 240) {
        raw_score += 5;
        positive_reasons.push(ScoreReason {
            id: "score-low-meeting-load".to_string(),
            label: "Meeting time stays below 4h per day".to_string(),
            points: 5,
            date: None,
            event_ids: Vec::new(),
        });
    }

    let score = raw_score.clamp(0, 100) as u8;
    HealthScore {
        score,
        grade: grade_for_score(score),
        positive_reasons,
        negative_reasons,
    }
}

fn has_meeting_free_day(busy_instances: &[EventInstance]) -> bool {
    if busy_instances.is_empty() {
        return true;
    }
    let mut days = busy_instances
        .iter()
        .map(|event| event.starts_at.date_naive())
        .collect::<Vec<_>>();
    days.sort();
    days.dedup();
    days.windows(2)
        .any(|pair| (pair[1] - pair[0]).num_days() > 1)
}

fn grade_for_score(score: u8) -> HealthGrade {
    match score {
        90..=100 => HealthGrade::Excellent,
        75..=89 => HealthGrade::Good,
        60..=74 => HealthGrade::Warning,
        40..=59 => HealthGrade::Poor,
        _ => HealthGrade::Critical,
    }
}
