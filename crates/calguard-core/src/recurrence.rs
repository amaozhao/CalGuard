use crate::error::CoreError;
use crate::interval::{overlaps, Interval};
use crate::model::{Event, EventInstance};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Frequency {
    Daily,
    Weekly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecurrenceRule {
    frequency: Frequency,
    interval: i64,
    count: Option<usize>,
    until: Option<DateTime<Utc>>,
}

pub fn expand_events(
    events: &[Event],
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
    max_per_event: usize,
) -> Vec<EventInstance> {
    let window = Interval::new(window_start, window_end);
    let mut instances = Vec::new();
    for event in events {
        if let Some(rule) = event.recurrence_rule.as_deref() {
            match parse_rrule(rule) {
                Ok(rule) => {
                    instances.extend(expand_recurring_event(event, &rule, &window, max_per_event))
                }
                Err(_) => {
                    let instance = instance_from_event(event, 0, event.starts_at, event.ends_at);
                    if overlaps(
                        &Interval::new(instance.starts_at, instance.ends_at),
                        &window,
                    ) {
                        instances.push(instance);
                    }
                }
            }
        } else {
            let instance = instance_from_event(event, 0, event.starts_at, event.ends_at);
            if overlaps(
                &Interval::new(instance.starts_at, instance.ends_at),
                &window,
            ) {
                instances.push(instance);
            }
        }
    }
    instances
        .sort_by_key(|instance| (instance.starts_at, instance.ends_at, instance.title.clone()));
    instances
}

fn expand_recurring_event(
    event: &Event,
    rule: &RecurrenceRule,
    window: &Interval,
    max_per_event: usize,
) -> Vec<EventInstance> {
    let mut instances = Vec::new();
    let duration = event.ends_at - event.starts_at;
    let step = match rule.frequency {
        Frequency::Daily => Duration::days(rule.interval),
        Frequency::Weekly => Duration::weeks(rule.interval),
    };
    if step <= Duration::zero() || duration <= Duration::zero() {
        return instances;
    }

    let mut starts_at = event.starts_at;
    let mut ordinal: usize = 0;
    if event.ends_at < window.starts_at {
        let diff = window.starts_at - event.ends_at;
        let skip = (diff.num_seconds() / step.num_seconds()).max(0) as usize;
        starts_at += step * skip as i32;
        ordinal += skip;
    }

    while instances.len() < max_per_event {
        if let Some(count) = rule.count {
            if ordinal >= count {
                break;
            }
        }
        if let Some(until) = rule.until {
            if starts_at > until {
                break;
            }
        }
        if starts_at >= window.ends_at && rule.count.is_none() {
            break;
        }

        let ends_at = starts_at + duration;
        let interval = Interval::new(starts_at, ends_at);
        let is_excluded = event.excluded_dates.contains(&starts_at);
        if !is_excluded && overlaps(&interval, window) {
            instances.push(instance_from_event(event, ordinal, starts_at, ends_at));
        }

        ordinal += 1;
        starts_at += step;
        if starts_at > window.ends_at && rule.count.is_none() && rule.until.is_none() {
            break;
        }
    }

    instances
}

fn instance_from_event(
    event: &Event,
    ordinal: usize,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
) -> EventInstance {
    EventInstance {
        id: format!("{}#{ordinal}", event.id),
        event_id: event.id.clone(),
        source_id: event.source_id.clone(),
        uid: event.uid.clone(),
        title: event.title.clone(),
        description: event.description.clone(),
        location: event.location.clone(),
        starts_at,
        ends_at,
        is_all_day: event.is_all_day,
        status: event.status,
        transparency: event.transparency,
    }
}

fn parse_rrule(raw: &str) -> Result<RecurrenceRule, CoreError> {
    let mut fields = HashMap::new();
    for part in raw.split(';') {
        let (key, value) = part
            .split_once('=')
            .ok_or_else(|| CoreError::InvalidRecurrenceRule(raw.to_string()))?;
        fields.insert(key.to_ascii_uppercase(), value.to_string());
    }

    let frequency = match fields.get("FREQ").map(|value| value.to_ascii_uppercase()) {
        Some(freq) if freq == "DAILY" => Frequency::Daily,
        Some(freq) if freq == "WEEKLY" => Frequency::Weekly,
        _ => return Err(CoreError::InvalidRecurrenceRule(raw.to_string())),
    };
    let interval = fields
        .get("INTERVAL")
        .map(|value| value.parse::<i64>())
        .transpose()
        .map_err(|_| CoreError::InvalidRecurrenceRule(raw.to_string()))?
        .unwrap_or(1)
        .max(1);
    let count = fields
        .get("COUNT")
        .map(|value| value.parse::<usize>())
        .transpose()
        .map_err(|_| CoreError::InvalidRecurrenceRule(raw.to_string()))?;
    let until = fields
        .get("UNTIL")
        .map(|value| parse_until(value))
        .transpose()?;

    Ok(RecurrenceRule {
        frequency,
        interval,
        count,
        until,
    })
}

fn parse_until(value: &str) -> Result<DateTime<Utc>, CoreError> {
    let source_id = "__rrule".to_string();
    let ics = format!(
        "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:until\nDTSTART:{value}\nDTEND:20990101T000000Z\nEND:VEVENT\nEND:VCALENDAR\n"
    );
    let result = crate::parser::parse_ics(&ics, &source_id);
    result
        .events
        .first()
        .map(|event| event.starts_at)
        .ok_or_else(|| CoreError::InvalidRecurrenceRule(format!("invalid UNTIL={value}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EventStatus, Transparency};
    use chrono::{TimeZone, Utc};

    fn dt(day: u32, hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, day, hour, 0, 0)
            .single()
            .unwrap()
    }

    fn event(rule: &str) -> Event {
        Event {
            id: "e1".to_string(),
            source_id: "s1".to_string(),
            uid: "u1".to_string(),
            title: "Standup".to_string(),
            description: None,
            location: None,
            starts_at: dt(1, 9),
            ends_at: dt(1, 10),
            timezone: None,
            is_all_day: false,
            status: EventStatus::Confirmed,
            transparency: Transparency::Opaque,
            recurrence_rule: Some(rule.to_string()),
            excluded_dates: Vec::new(),
            recurrence_id: None,
        }
    }

    #[test]
    fn expands_daily_count_inside_window() {
        let instances = expand_events(&[event("FREQ=DAILY;COUNT=3")], dt(1, 0), dt(8, 0), 1000);

        assert_eq!(instances.len(), 3);
        assert_eq!(instances[2].starts_at, dt(3, 9));
    }

    #[test]
    fn expands_weekly_until_inside_window() {
        let instances = expand_events(
            &[event("FREQ=WEEKLY;UNTIL=20260615T090000Z")],
            dt(1, 0),
            dt(30, 0),
            1000,
        );

        assert_eq!(instances.len(), 3);
        assert_eq!(instances[2].starts_at, dt(15, 9));
    }

    #[test]
    fn excludes_matching_exdate_and_caps_infinite_rule() {
        let mut recurring = event("FREQ=DAILY");
        recurring.excluded_dates = vec![dt(2, 9)];
        let instances = expand_events(&[recurring], dt(1, 0), dt(5, 0), 2);

        assert_eq!(instances.len(), 2);
        assert_eq!(instances[0].starts_at, dt(1, 9));
        assert_eq!(instances[1].starts_at, dt(3, 9));
    }
}
