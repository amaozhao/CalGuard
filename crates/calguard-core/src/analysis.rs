use crate::interval::{merge_intervals, overlaps, subtract_intervals, Interval};
use crate::model::*;
use crate::recurrence::expand_events;
use crate::scoring::score_health;
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

pub fn analyze_calendar(input: AnalysisInput) -> AnalysisReport {
    let range_days = input.settings.normalized_range_days();
    let period_start = input.generated_at;
    let period_end = period_start + Duration::days(range_days as i64);
    let enabled_sources = input.enabled_source_ids.into_iter().collect::<HashSet<_>>();
    let ignored_hashes = input
        .ignored_conflict_hashes
        .into_iter()
        .collect::<HashSet<_>>();

    let source_filtered_events = input
        .events
        .into_iter()
        .filter(|event| enabled_sources.contains(&event.source_id))
        .collect::<Vec<_>>();
    let instances = expand_events(
        &source_filtered_events,
        period_start,
        period_end,
        input.settings.max_recurring_instances,
    );
    let busy_instances = instances
        .iter()
        .filter(|instance| instance.is_busy(&input.settings))
        .cloned()
        .collect::<Vec<_>>();

    let conflicts = detect_conflicts(&busy_instances, &ignored_hashes);
    let free_blocks =
        calculate_free_blocks(&busy_instances, period_start, period_end, &input.settings);
    let overloaded_days = detect_overloaded_days(
        &busy_instances,
        &free_blocks,
        period_start,
        range_days,
        &input.settings,
    );
    let focus_metrics = calculate_focus_metrics(
        &busy_instances,
        &free_blocks,
        period_start,
        range_days,
        &input.settings,
    );
    let suggestions = build_suggestions(&conflicts, &free_blocks, &overloaded_days);
    let score = score_health(
        &conflicts,
        &free_blocks,
        &overloaded_days,
        &busy_instances,
        &input.settings,
        &focus_metrics,
    );

    AnalysisReport {
        schema_version: "calguard.analysis.v1".to_string(),
        period_start,
        period_end,
        generated_at: input.generated_at,
        score,
        conflicts,
        free_blocks,
        overloaded_days,
        focus_metrics,
        suggestions,
    }
}

fn detect_conflicts(
    instances: &[EventInstance],
    ignored_hashes: &HashSet<String>,
) -> Vec<Conflict> {
    let mut busy = instances.to_vec();
    busy.sort_by_key(|event| (event.starts_at, event.ends_at, event.id.clone()));

    let mut conflicts = Vec::new();
    let mut seen = HashSet::new();
    let mut covered_pairs = HashSet::new();

    for i in 0..busy.len() {
        let anchor = &busy[i];
        let anchor_interval = Interval::new(anchor.starts_at, anchor.ends_at);
        let mut group = vec![anchor.clone()];
        for candidate in busy.iter().skip(i + 1) {
            if candidate.starts_at >= anchor.ends_at {
                break;
            }
            let candidate_interval = Interval::new(candidate.starts_at, candidate.ends_at);
            if overlaps(&anchor_interval, &candidate_interval) {
                group.push(candidate.clone());
            }
        }

        if group.len() < 2 {
            continue;
        }

        let starts_at = group
            .iter()
            .map(|event| event.starts_at)
            .max()
            .unwrap_or(anchor.starts_at);
        let ends_at = group
            .iter()
            .map(|event| event.ends_at)
            .min()
            .unwrap_or(anchor.ends_at);
        if starts_at >= ends_at {
            continue;
        }

        let mut ids = group
            .iter()
            .map(|event| event.id.clone())
            .collect::<Vec<_>>();
        ids.sort();
        if group.len() >= 3 {
            for left in 0..ids.len() {
                for right in (left + 1)..ids.len() {
                    covered_pairs.insert(format!("{}|{}", ids[left], ids[right]));
                }
            }
        } else {
            let pair = format!("{}|{}", ids[0], ids[1]);
            if covered_pairs.contains(&pair) {
                continue;
            }
        }

        let identity = format!(
            "{}:{}:{}",
            starts_at.timestamp(),
            ends_at.timestamp(),
            ids.join("|")
        );
        if !seen.insert(identity.clone()) {
            continue;
        }

        let overlap_minutes = (ends_at - starts_at).num_minutes().max(1);
        let severity = if group.len() >= 3 {
            ConflictSeverity::Critical
        } else if overlap_minutes <= 10 {
            ConflictSeverity::Low
        } else if overlap_minutes <= 30 {
            ConflictSeverity::Medium
        } else {
            ConflictSeverity::High
        };
        let ignore_hash = stable_hash(&format!("conflict:{identity}"));
        let ignored = ignored_hashes.contains(&ignore_hash);

        conflicts.push(Conflict {
            id: ignore_hash.clone(),
            event_ids: ids,
            events: group,
            starts_at,
            ends_at,
            overlap_minutes,
            severity,
            reason: format!("{overlap_minutes} minute overlap"),
            ignored,
            ignore_hash,
        });
    }

    conflicts.sort_by_key(|conflict| (conflict.starts_at, conflict.ends_at));
    conflicts
}

fn calculate_free_blocks(
    busy_instances: &[EventInstance],
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    settings: &AnalysisSettings,
) -> Vec<FreeBlock> {
    let mut blocks = Vec::new();
    for date in dates_in_range(period_start.date_naive(), settings.normalized_range_days()) {
        if !settings.is_workday_number(weekday_number(date)) {
            continue;
        }
        let work_window = day_interval(date, settings.work_start, settings.work_end);
        let analysis_window = Interval::new(period_start, period_end);
        let Some(work_window) = work_window.clipped_to(&analysis_window) else {
            continue;
        };
        let busy = busy_instances
            .iter()
            .filter_map(|event| {
                let event_interval = Interval::new(event.starts_at, event.ends_at);
                event_interval.clipped_to(&work_window)
            })
            .collect::<Vec<_>>();
        let free = subtract_intervals(work_window, &busy);
        for interval in free {
            blocks.push(FreeBlock {
                id: stable_hash(&format!("free:{}:{}", interval.starts_at, interval.ends_at)),
                starts_at: interval.starts_at,
                ends_at: interval.ends_at,
                duration_minutes: interval.duration_minutes(),
                block_type: classify_free_block(interval, date, settings),
            });
        }
    }
    blocks
}

fn classify_free_block(
    interval: Interval,
    date: NaiveDate,
    settings: &AnalysisSettings,
) -> FreeBlockType {
    let lunch = day_interval(date, settings.lunch_start, settings.lunch_end);
    let duration = interval.duration_minutes();
    if overlaps(&interval, &lunch)
        && duration <= (settings.lunch_end - settings.lunch_start).num_minutes()
    {
        FreeBlockType::Lunch
    } else if duration >= settings.min_focus_minutes {
        FreeBlockType::DeepWork
    } else if duration >= settings.min_short_gap_minutes {
        FreeBlockType::ShortGap
    } else {
        FreeBlockType::MicroGap
    }
}

fn detect_overloaded_days(
    busy_instances: &[EventInstance],
    free_blocks: &[FreeBlock],
    period_start: DateTime<Utc>,
    range_days: u32,
    settings: &AnalysisSettings,
) -> Vec<OverloadedDay> {
    let mut days = Vec::new();
    for date in dates_in_range(period_start.date_naive(), range_days) {
        if !settings.is_workday_number(weekday_number(date)) {
            continue;
        }
        let work_window = day_interval(date, settings.work_start, settings.work_end);
        let lunch_window = day_interval(date, settings.lunch_start, settings.lunch_end);
        let day_window = day_interval(
            date,
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
        );
        let day_events = busy_instances
            .iter()
            .filter(|event| overlaps(&Interval::new(event.starts_at, event.ends_at), &day_window))
            .cloned()
            .collect::<Vec<_>>();
        let meeting_count = day_events.len();
        let meeting_minutes = merged_minutes_in_window(&day_events, work_window);
        let has_deep_work = free_blocks.iter().any(|block| {
            block.starts_at.date_naive() == date && block.block_type == FreeBlockType::DeepWork
        });
        let lunch_covered = day_events.iter().any(|event| {
            overlaps(
                &Interval::new(event.starts_at, event.ends_at),
                &lunch_window,
            )
        });
        let has_late_meeting = day_events.iter().any(|event| {
            event.ends_at
                > Utc.from_utc_datetime(&date.and_time(NaiveTime::from_hms_opt(19, 0, 0).unwrap()))
        });
        let consecutive_groups = count_consecutive_groups(&day_events);

        let mut reasons = Vec::new();
        if meeting_minutes > settings.severe_overload_meeting_minutes {
            reasons.push(format!("{}h of meetings", meeting_minutes as f32 / 60.0));
        } else if meeting_minutes > settings.overload_meeting_minutes {
            reasons.push(format!(
                "more than {}h of meetings",
                settings.overload_meeting_minutes / 60
            ));
        }
        if meeting_count > settings.overload_meeting_count {
            reasons.push(format!("{meeting_count} meetings"));
        }
        if consecutive_groups > 0 {
            reasons.push("more than 3 consecutive meetings".to_string());
        }
        if lunch_covered {
            reasons.push("lunch is covered by meetings".to_string());
        }
        if !has_deep_work {
            reasons.push("no deep work block".to_string());
        }
        if has_late_meeting {
            reasons.push("meeting after 19:00".to_string());
        }

        if reasons.is_empty() {
            continue;
        }

        let status = if meeting_minutes > settings.severe_overload_meeting_minutes {
            OverloadStatus::SeverelyOverloaded
        } else if meeting_minutes > settings.overload_meeting_minutes
            || meeting_count > settings.overload_meeting_count
        {
            OverloadStatus::Overloaded
        } else if has_late_meeting {
            OverloadStatus::BoundaryRisk
        } else {
            OverloadStatus::Risk
        };

        days.push(OverloadedDay {
            date: date.to_string(),
            status,
            meeting_minutes,
            meeting_count,
            reasons,
        });
    }
    days
}

fn calculate_focus_metrics(
    busy_instances: &[EventInstance],
    free_blocks: &[FreeBlock],
    period_start: DateTime<Utc>,
    range_days: u32,
    settings: &AnalysisSettings,
) -> FocusMetrics {
    let deep_work_blocks = free_blocks
        .iter()
        .filter(|block| block.block_type == FreeBlockType::DeepWork)
        .count();
    let focus_minutes = free_blocks
        .iter()
        .filter(|block| block.block_type == FreeBlockType::DeepWork)
        .map(|block| block.duration_minutes)
        .sum::<i64>();
    let longest_free_block_minutes = free_blocks
        .iter()
        .map(|block| block.duration_minutes)
        .max()
        .unwrap_or(0);
    let no_focus_days = dates_in_range(period_start.date_naive(), range_days)
        .into_iter()
        .filter(|date| settings.is_workday_number(weekday_number(*date)))
        .filter(|date| {
            !free_blocks.iter().any(|block| {
                block.starts_at.date_naive() == *date && block.block_type == FreeBlockType::DeepWork
            })
        })
        .map(|date| date.to_string())
        .collect::<Vec<_>>();

    let total_work_minutes = dates_in_range(period_start.date_naive(), range_days)
        .into_iter()
        .filter(|date| settings.is_workday_number(weekday_number(*date)))
        .map(|_| (settings.work_end - settings.work_start).num_minutes())
        .sum::<i64>()
        .max(1);
    let meeting_minutes = dates_in_range(period_start.date_naive(), range_days)
        .into_iter()
        .filter(|date| settings.is_workday_number(weekday_number(*date)))
        .map(|date| {
            merged_minutes_in_window(
                busy_instances,
                day_interval(date, settings.work_start, settings.work_end),
            )
        })
        .sum::<i64>();
    let meeting_density_percent =
        ((meeting_minutes * 100) / total_work_minutes).clamp(0, 100) as u8;
    let micro_gaps = free_blocks
        .iter()
        .filter(|block| block.block_type == FreeBlockType::MicroGap)
        .count() as i64;
    let fragmentation_score = (micro_gaps * 10
        + no_focus_days.len() as i64 * 8
        + (settings.min_focus_minutes - longest_free_block_minutes).max(0) / 2
        + meeting_density_percent as i64 / 2)
        .clamp(0, 100) as u8;

    FocusMetrics {
        deep_work_blocks,
        focus_minutes,
        longest_free_block_minutes,
        no_focus_days,
        meeting_density_percent,
        fragmentation_score,
    }
}

fn build_suggestions(
    conflicts: &[Conflict],
    free_blocks: &[FreeBlock],
    overloaded_days: &[OverloadedDay],
) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();
    for conflict in conflicts
        .iter()
        .filter(|conflict| !conflict.ignored)
        .take(5)
    {
        let title = conflict
            .events
            .first()
            .map(|event| event.title.as_str())
            .unwrap_or("one event");
        suggestions.push(Suggestion {
            id: stable_hash(&format!("suggestion:move:{}", conflict.id)),
            kind: SuggestionKind::MoveEvent,
            text: format!(
                "Move {title} away from {}",
                conflict.starts_at.format("%Y-%m-%d %H:%M")
            ),
            reason: format!(
                "It overlaps with {} event(s) for {} minutes.",
                conflict.events.len().saturating_sub(1),
                conflict.overlap_minutes
            ),
            date: Some(conflict.starts_at.date_naive().to_string()),
            event_ids: conflict.event_ids.clone(),
            free_block_id: None,
            ignored: false,
        });
    }

    for block in free_blocks
        .iter()
        .filter(|block| block.block_type == FreeBlockType::DeepWork)
        .take(5)
    {
        suggestions.push(Suggestion {
            id: stable_hash(&format!("suggestion:focus:{}", block.id)),
            kind: SuggestionKind::AddFocusBlock,
            text: format!(
                "Protect {}-{} as Focus Time.",
                block.starts_at.format("%a %H:%M"),
                block.ends_at.format("%H:%M")
            ),
            reason: format!(
                "This is a {} minute deep work block.",
                block.duration_minutes
            ),
            date: Some(block.starts_at.date_naive().to_string()),
            event_ids: Vec::new(),
            free_block_id: Some(block.id.clone()),
            ignored: false,
        });
    }

    for day in overloaded_days.iter().take(3) {
        suggestions.push(Suggestion {
            id: stable_hash(&format!("suggestion:buffer:{}", day.date)),
            kind: SuggestionKind::AddBuffer,
            text: format!("Review buffers on {}.", day.date),
            reason: day.reasons.join(", "),
            date: Some(day.date.clone()),
            event_ids: Vec::new(),
            free_block_id: None,
            ignored: false,
        });
    }

    suggestions.truncate(10);
    suggestions
}

fn merged_minutes_in_window(events: &[EventInstance], window: Interval) -> i64 {
    let intervals = events
        .iter()
        .filter_map(|event| Interval::new(event.starts_at, event.ends_at).clipped_to(&window))
        .collect::<Vec<_>>();
    merge_intervals(&intervals)
        .iter()
        .map(Interval::duration_minutes)
        .sum()
}

fn count_consecutive_groups(events: &[EventInstance]) -> usize {
    let mut sorted = events.to_vec();
    sorted.sort_by_key(|event| (event.starts_at, event.ends_at));
    let mut groups = 0;
    let mut streak = 1;
    let mut last_end = None;
    for event in sorted {
        if let Some(end) = last_end {
            if event.starts_at <= end {
                streak += 1;
            } else {
                if streak > 3 {
                    groups += 1;
                }
                streak = 1;
            }
        }
        last_end = Some(event.ends_at);
    }
    if streak > 3 {
        groups += 1;
    }
    groups
}

fn day_interval(date: NaiveDate, start: NaiveTime, end: NaiveTime) -> Interval {
    Interval::new(
        Utc.from_utc_datetime(&date.and_time(start)),
        Utc.from_utc_datetime(&date.and_time(end)),
    )
}

fn dates_in_range(start_date: NaiveDate, range_days: u32) -> Vec<NaiveDate> {
    (0..range_days)
        .map(|offset| start_date + Duration::days(offset as i64))
        .collect()
}

fn weekday_number(date: NaiveDate) -> u32 {
    date.weekday().num_days_from_monday() + 1
}

pub(crate) fn stable_hash(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    digest[..12]
        .iter()
        .fold(String::with_capacity(24), |mut output, byte| {
            output.push_str(&format!("{byte:02x}"));
            output
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EventStatus, Transparency};
    use chrono::{TimeZone, Utc};

    fn dt(day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, day, hour, minute, 0)
            .single()
            .unwrap()
    }

    fn event(id: &str, source: &str, start_h: u32, end_h: u32) -> Event {
        Event {
            id: id.to_string(),
            source_id: source.to_string(),
            uid: id.to_string(),
            title: id.to_string(),
            description: None,
            location: None,
            starts_at: dt(1, start_h, 0),
            ends_at: dt(1, end_h, 0),
            timezone: None,
            is_all_day: false,
            status: EventStatus::Confirmed,
            transparency: Transparency::Opaque,
            recurrence_rule: None,
            excluded_dates: Vec::new(),
            recurrence_id: None,
        }
    }

    #[test]
    fn detects_conflicts_and_free_time() {
        let report = analyze_calendar(AnalysisInput {
            events: vec![event("a", "s1", 10, 11), event("b", "s2", 10, 12)],
            settings: AnalysisSettings {
                range_days: 1,
                ..Default::default()
            },
            enabled_source_ids: vec!["s1".to_string(), "s2".to_string()],
            ignored_conflict_hashes: Vec::new(),
            generated_at: dt(1, 0, 0),
        });

        assert_eq!(report.conflicts.len(), 1);
        assert_eq!(report.conflicts[0].overlap_minutes, 60);
        assert!(report
            .free_blocks
            .iter()
            .any(|block| block.block_type == FreeBlockType::DeepWork));
    }

    #[test]
    fn disabled_sources_do_not_participate() {
        let report = analyze_calendar(AnalysisInput {
            events: vec![event("a", "s1", 10, 11), event("b", "s2", 10, 12)],
            settings: AnalysisSettings {
                range_days: 1,
                ..Default::default()
            },
            enabled_source_ids: vec!["s1".to_string()],
            ignored_conflict_hashes: Vec::new(),
            generated_at: dt(1, 0, 0),
        });

        assert!(report.conflicts.is_empty());
    }

    #[test]
    fn empty_enabled_sources_exclude_all_events() {
        let report = analyze_calendar(AnalysisInput {
            events: vec![event("a", "s1", 10, 11)],
            settings: AnalysisSettings {
                range_days: 1,
                ..Default::default()
            },
            enabled_source_ids: Vec::new(),
            ignored_conflict_hashes: Vec::new(),
            generated_at: dt(1, 0, 0),
        });

        assert!(report.conflicts.is_empty());
        assert_eq!(report.focus_metrics.meeting_density_percent, 0);
    }
}
