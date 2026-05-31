pub mod analysis;
pub mod error;
pub mod interval;
pub mod model;
pub mod parser;
pub mod recurrence;
pub mod report;
pub mod scoring;

pub use analysis::analyze_calendar;
pub use error::{CoreError, ParseIssue};
pub use interval::{merge_intervals, overlaps, subtract_intervals, Interval};
pub use model::*;
pub use parser::parse_ics;
pub use recurrence::expand_events;
pub use report::{render_json_report, render_markdown_report};
