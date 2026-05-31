use crate::error::{CoreError, ParseIssue};
use crate::model::{CalendarSourceId, Event, EventStatus, Transparency};
use chrono::{DateTime, Duration, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IcsParseResult {
    pub events: Vec<Event>,
    pub issues: Vec<ParseIssue>,
}

#[derive(Debug, Clone)]
struct ParsedProperty {
    line: usize,
    name: String,
    params: HashMap<String, String>,
    value: String,
}

pub fn parse_ics(input: &str, source_id: &CalendarSourceId) -> IcsParseResult {
    let lines = unfold_lines(input);
    let mut issues = Vec::new();
    let mut events = Vec::new();
    let mut current: Option<Vec<(usize, String)>> = None;

    for (line_no, line) in lines {
        let upper = line.to_ascii_uppercase();
        match upper.as_str() {
            "BEGIN:VEVENT" => {
                if current.is_some() {
                    issues.push(ParseIssue::new(
                        Some(line_no),
                        "VEVENT",
                        "nested VEVENT is not supported",
                    ));
                }
                current = Some(vec![(line_no, line)]);
            }
            "END:VEVENT" => {
                if let Some(mut event_lines) = current.take() {
                    event_lines.push((line_no, line));
                    match parse_event(&event_lines, source_id) {
                        Ok((event, event_issues)) => {
                            events.push(event);
                            issues.extend(event_issues);
                        }
                        Err(issue) => issues.push(issue),
                    }
                } else {
                    issues.push(ParseIssue::new(
                        Some(line_no),
                        "VEVENT",
                        "END:VEVENT without BEGIN:VEVENT",
                    ));
                }
            }
            _ => {
                if let Some(event_lines) = current.as_mut() {
                    event_lines.push((line_no, line));
                }
            }
        }
    }

    if let Some(event_lines) = current {
        let line = event_lines.first().map(|(line_no, _)| *line_no);
        issues.push(ParseIssue::new(
            line,
            "VEVENT",
            "VEVENT was not closed with END:VEVENT",
        ));
    }

    IcsParseResult { events, issues }
}

fn unfold_lines(input: &str) -> Vec<(usize, String)> {
    let mut unfolded: Vec<(usize, String)> = Vec::new();
    for (idx, raw_line) in input.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim_end_matches('\r');
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some((_, previous)) = unfolded.last_mut() {
                previous.push_str(&line[1..]);
            }
        } else {
            unfolded.push((line_no, line.to_string()));
        }
    }
    unfolded
}

fn parse_event(
    lines: &[(usize, String)],
    source_id: &CalendarSourceId,
) -> Result<(Event, Vec<ParseIssue>), ParseIssue> {
    let mut issues = Vec::new();
    let mut properties = Vec::new();

    for (line_no, line) in lines {
        if line.eq_ignore_ascii_case("BEGIN:VEVENT") || line.eq_ignore_ascii_case("END:VEVENT") {
            continue;
        }
        match parse_property(*line_no, line) {
            Ok(property) => properties.push(property),
            Err(issue) => issues.push(issue),
        }
    }

    let uid = required_value(&properties, "UID")
        .ok_or_else(|| ParseIssue::new(first_line(lines), "VEVENT", "missing required UID"))?;
    let title = value(&properties, "SUMMARY").unwrap_or_else(|| "Untitled Event".to_string());
    let description = value(&properties, "DESCRIPTION");
    let location = value(&properties, "LOCATION");
    let status = match value(&properties, "STATUS")
        .unwrap_or_else(|| "CONFIRMED".to_string())
        .to_ascii_uppercase()
        .as_str()
    {
        "CANCELLED" => EventStatus::Cancelled,
        "TENTATIVE" => EventStatus::Tentative,
        _ => EventStatus::Confirmed,
    };
    let transparency = match value(&properties, "TRANSP")
        .unwrap_or_else(|| "OPAQUE".to_string())
        .to_ascii_uppercase()
        .as_str()
    {
        "TRANSPARENT" => Transparency::Transparent,
        _ => Transparency::Opaque,
    };

    let dtstart = property(&properties, "DTSTART").ok_or_else(|| {
        ParseIssue::new(first_line(lines), uid.clone(), "missing required DTSTART")
    })?;
    let (starts_at, is_all_day, timezone) = parse_datetime_property(dtstart)
        .map_err(|err| ParseIssue::new(Some(dtstart.line), uid.clone(), err.to_string()))?;

    let ends_at = if let Some(dtend) = property(&properties, "DTEND") {
        let (ends_at, _, _) = parse_datetime_property(dtend)
            .map_err(|err| ParseIssue::new(Some(dtend.line), uid.clone(), err.to_string()))?;
        ends_at
    } else if let Some(duration_prop) = property(&properties, "DURATION") {
        starts_at
            + parse_duration(&duration_prop.value).map_err(|err| {
                ParseIssue::new(Some(duration_prop.line), uid.clone(), err.to_string())
            })?
    } else if is_all_day {
        starts_at + Duration::days(1)
    } else {
        return Err(ParseIssue::new(
            Some(dtstart.line),
            uid,
            "VEVENT must include DTEND or DURATION unless it is an all-day event",
        ));
    };

    if starts_at >= ends_at {
        return Err(ParseIssue::new(
            Some(dtstart.line),
            uid,
            "VEVENT starts_at must be before ends_at",
        ));
    }

    let excluded_dates = properties
        .iter()
        .filter(|property| property.name == "EXDATE")
        .flat_map(|property| parse_exdates(property, &uid, &mut issues))
        .collect::<Vec<_>>();
    let recurrence_id = property(&properties, "RECURRENCE-ID").and_then(|property| {
        match parse_datetime_property(property) {
            Ok((date, _, _)) => Some(date),
            Err(err) => {
                issues.push(ParseIssue::new(
                    Some(property.line),
                    uid.clone(),
                    err.to_string(),
                ));
                None
            }
        }
    });

    let event = Event {
        id: stable_event_id(source_id, &uid, starts_at),
        source_id: source_id.clone(),
        uid,
        title,
        description,
        location,
        starts_at,
        ends_at,
        timezone,
        is_all_day,
        status,
        transparency,
        recurrence_rule: value(&properties, "RRULE"),
        excluded_dates,
        recurrence_id,
    };

    Ok((event, issues))
}

fn parse_property(line: usize, raw: &str) -> Result<ParsedProperty, ParseIssue> {
    let (left, raw_value) = raw
        .split_once(':')
        .ok_or_else(|| ParseIssue::new(Some(line), raw, "property line is missing ':'"))?;
    let mut left_parts = left.split(';');
    let name = left_parts
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_uppercase();
    if name.is_empty() {
        return Err(ParseIssue::new(Some(line), raw, "property name is empty"));
    }

    let mut params = HashMap::new();
    for param in left_parts {
        if let Some((key, value)) = param.split_once('=') {
            params.insert(
                key.trim().to_ascii_uppercase(),
                value.trim_matches('"').to_string(),
            );
        }
    }

    Ok(ParsedProperty {
        line,
        name,
        params,
        value: unescape_value(raw_value),
    })
}

fn unescape_value(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') | Some('N') => output.push('\n'),
                Some(',') => output.push(','),
                Some(';') => output.push(';'),
                Some('\\') => output.push('\\'),
                Some(other) => {
                    output.push('\\');
                    output.push(other);
                }
                None => output.push('\\'),
            }
        } else {
            output.push(ch);
        }
    }
    output
}

fn property<'a>(properties: &'a [ParsedProperty], name: &str) -> Option<&'a ParsedProperty> {
    properties.iter().find(|property| property.name == name)
}

fn value(properties: &[ParsedProperty], name: &str) -> Option<String> {
    property(properties, name).map(|property| property.value.clone())
}

fn required_value(properties: &[ParsedProperty], name: &str) -> Option<String> {
    value(properties, name).filter(|value| !value.trim().is_empty())
}

fn first_line(lines: &[(usize, String)]) -> Option<usize> {
    lines.first().map(|(line, _)| *line)
}

fn parse_datetime_property(
    property: &ParsedProperty,
) -> Result<(DateTime<Utc>, bool, Option<String>), CoreError> {
    let value = property.value.trim();
    let value_type = property
        .params
        .get("VALUE")
        .map(|value| value.to_ascii_uppercase())
        .unwrap_or_default();
    if value_type == "DATE" || (value.len() == 8 && value.chars().all(|ch| ch.is_ascii_digit())) {
        let date = NaiveDate::parse_from_str(value, "%Y%m%d")
            .map_err(|_| CoreError::InvalidDateTime(value.to_string()))?;
        let naive = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| CoreError::InvalidDateTime(value.to_string()))?;
        return Ok((
            Utc.from_utc_datetime(&naive),
            true,
            property.params.get("TZID").cloned(),
        ));
    }

    let without_z = value.strip_suffix('Z').unwrap_or(value);
    let naive = parse_datetime_without_timezone(without_z)?;
    if value.ends_with('Z') {
        return Ok((
            Utc.from_utc_datetime(&naive),
            false,
            Some("UTC".to_string()),
        ));
    }

    if let Some(tzid) = property.params.get("TZID") {
        let timezone =
            Tz::from_str(tzid).map_err(|_| CoreError::UnsupportedTimezone(tzid.clone()))?;
        let local = match timezone.from_local_datetime(&naive) {
            LocalResult::Single(date) => date,
            LocalResult::Ambiguous(earliest, _) => earliest,
            LocalResult::None => return Err(CoreError::InvalidDateTime(value.to_string())),
        };
        return Ok((local.with_timezone(&Utc), false, Some(tzid.clone())));
    }

    Ok((Utc.from_utc_datetime(&naive), false, None))
}

fn parse_datetime_without_timezone(value: &str) -> Result<NaiveDateTime, CoreError> {
    for format in ["%Y%m%dT%H%M%S", "%Y%m%dT%H%M"] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(value, format) {
            return Ok(naive);
        }
    }
    Err(CoreError::InvalidDateTime(value.to_string()))
}

fn parse_duration(value: &str) -> Result<Duration, CoreError> {
    let mut chars = value.chars();
    if chars.next() != Some('P') {
        return Err(CoreError::InvalidDateTime(format!(
            "invalid duration {value}"
        )));
    }

    let mut in_time = false;
    let mut number = String::new();
    let mut seconds = 0_i64;

    for ch in chars {
        if ch == 'T' {
            in_time = true;
            continue;
        }
        if ch.is_ascii_digit() {
            number.push(ch);
            continue;
        }
        let amount = number
            .parse::<i64>()
            .map_err(|_| CoreError::InvalidDateTime(format!("invalid duration {value}")))?;
        number.clear();
        match ch {
            'D' if !in_time => seconds += amount * 86_400,
            'H' if in_time => seconds += amount * 3_600,
            'M' if in_time => seconds += amount * 60,
            'S' if in_time => seconds += amount,
            _ => {
                return Err(CoreError::InvalidDateTime(format!(
                    "invalid duration {value}"
                )))
            }
        }
    }

    if !number.is_empty() || seconds <= 0 {
        return Err(CoreError::InvalidDateTime(format!(
            "invalid duration {value}"
        )));
    }

    Ok(Duration::seconds(seconds))
}

fn parse_exdates(
    property: &ParsedProperty,
    uid: &str,
    issues: &mut Vec<ParseIssue>,
) -> Vec<DateTime<Utc>> {
    property
        .value
        .split(',')
        .filter_map(|value| {
            let mut exdate_property = property.clone();
            exdate_property.value = value.trim().to_string();
            match parse_datetime_property(&exdate_property) {
                Ok((date, _, _)) => Some(date),
                Err(err) => {
                    issues.push(ParseIssue::new(
                        Some(property.line),
                        uid.to_string(),
                        err.to_string(),
                    ));
                    None
                }
            }
        })
        .collect()
}

fn stable_event_id(source_id: &str, uid: &str, starts_at: DateTime<Utc>) -> String {
    format!("{source_id}:{uid}:{}", starts_at.timestamp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_event() {
        let source_id = "source-a".to_string();
        let result = parse_ics(
            "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:1\r\nSUMMARY:Planning\r\nDTSTART:20260601T090000Z\r\nDTEND:20260601T100000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
            &source_id,
        );

        assert!(result.issues.is_empty(), "{:?}", result.issues);
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].title, "Planning");
        assert!(!result.events[0].is_all_day);
    }

    #[test]
    fn parses_all_day_duration_transparency_and_cancelled() {
        let source_id = "source-a".to_string();
        let result = parse_ics(
            "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:2\nSUMMARY:Holiday\nDTSTART;VALUE=DATE:20260605\nDURATION:P1D\nSTATUS:CANCELLED\nTRANSP:TRANSPARENT\nEND:VEVENT\nEND:VCALENDAR\n",
            &source_id,
        );

        assert!(result.issues.is_empty(), "{:?}", result.issues);
        let event = &result.events[0];
        assert!(event.is_all_day);
        assert_eq!(event.status, EventStatus::Cancelled);
        assert_eq!(event.transparency, Transparency::Transparent);
    }

    #[test]
    fn keeps_single_bad_event_from_failing_file() {
        let source_id = "source-a".to_string();
        let result = parse_ics(
            "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:bad\nDTSTART:bad\nDTEND:20260601T100000Z\nEND:VEVENT\nBEGIN:VEVENT\nUID:ok\nSUMMARY:OK\nDTSTART:20260601T110000Z\nDTEND:20260601T120000Z\nEND:VEVENT\nEND:VCALENDAR\n",
            &source_id,
        );

        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].uid, "ok");
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].line, Some(4));
    }
}
