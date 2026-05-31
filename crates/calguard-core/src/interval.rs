use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Interval {
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

impl Interval {
    pub fn new(starts_at: DateTime<Utc>, ends_at: DateTime<Utc>) -> Self {
        Self { starts_at, ends_at }
    }

    pub fn duration_minutes(&self) -> i64 {
        (self.ends_at - self.starts_at).num_minutes()
    }

    pub fn is_valid(&self) -> bool {
        self.starts_at < self.ends_at
    }

    pub fn clipped_to(&self, window: &Interval) -> Option<Self> {
        let starts_at = self.starts_at.max(window.starts_at);
        let ends_at = self.ends_at.min(window.ends_at);
        (starts_at < ends_at).then_some(Self { starts_at, ends_at })
    }
}

pub fn overlaps(a: &Interval, b: &Interval) -> bool {
    a.starts_at < b.ends_at && b.starts_at < a.ends_at
}

pub fn merge_intervals(intervals: &[Interval]) -> Vec<Interval> {
    let mut sorted = intervals
        .iter()
        .copied()
        .filter(Interval::is_valid)
        .collect::<Vec<_>>();
    sorted.sort_by_key(|interval| (interval.starts_at, interval.ends_at));

    let mut merged: Vec<Interval> = Vec::new();
    for interval in sorted {
        if let Some(last) = merged.last_mut() {
            if interval.starts_at <= last.ends_at {
                last.ends_at = last.ends_at.max(interval.ends_at);
                continue;
            }
        }
        merged.push(interval);
    }
    merged
}

pub fn subtract_intervals(window: Interval, busy_intervals: &[Interval]) -> Vec<Interval> {
    if !window.is_valid() {
        return Vec::new();
    }

    let clipped_busy = busy_intervals
        .iter()
        .filter_map(|interval| interval.clipped_to(&window))
        .collect::<Vec<_>>();
    let merged_busy = merge_intervals(&clipped_busy);

    let mut cursor = window.starts_at;
    let mut free = Vec::new();
    for busy in merged_busy {
        if cursor < busy.starts_at {
            free.push(Interval::new(cursor, busy.starts_at));
        }
        cursor = cursor.max(busy.ends_at);
    }

    if cursor < window.ends_at {
        free.push(Interval::new(cursor, window.ends_at));
    }

    free
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn dt(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, hour, minute, 0)
            .single()
            .unwrap()
    }

    #[test]
    fn overlap_excludes_touching_boundaries() {
        let a = Interval::new(dt(9, 0), dt(10, 0));
        let b = Interval::new(dt(10, 0), dt(11, 0));
        let c = Interval::new(dt(9, 30), dt(10, 30));

        assert!(!overlaps(&a, &b));
        assert!(overlaps(&a, &c));
    }

    #[test]
    fn merge_intervals_coalesces_touching_and_overlapping_ranges() {
        let merged = merge_intervals(&[
            Interval::new(dt(10, 0), dt(11, 0)),
            Interval::new(dt(9, 0), dt(10, 0)),
            Interval::new(dt(12, 0), dt(13, 0)),
            Interval::new(dt(12, 30), dt(14, 0)),
        ]);

        assert_eq!(
            merged,
            vec![
                Interval::new(dt(9, 0), dt(11, 0)),
                Interval::new(dt(12, 0), dt(14, 0))
            ]
        );
    }

    #[test]
    fn subtract_intervals_returns_free_blocks_inside_window() {
        let free = subtract_intervals(
            Interval::new(dt(9, 0), dt(18, 0)),
            &[
                Interval::new(dt(10, 0), dt(11, 0)),
                Interval::new(dt(13, 0), dt(14, 30)),
            ],
        );

        assert_eq!(
            free,
            vec![
                Interval::new(dt(9, 0), dt(10, 0)),
                Interval::new(dt(11, 0), dt(13, 0)),
                Interval::new(dt(14, 30), dt(18, 0)),
            ]
        );
    }
}
