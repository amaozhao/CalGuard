# CalGuard MVP Task Plan and Review Matrix

This plan turns `docs/PRD.md` into implementation tasks that can be checked
against the PRD acceptance list. P0 scope is intentionally read-only,
local-first, and static-export friendly.

## Architecture Decisions Reviewed

| Area | Decision | PRD Alignment |
| --- | --- | --- |
| Desktop shell | Tauri v2 with `frontendDist = "../out"` | Matches static frontend requirement |
| Frontend | Next.js App Router + React + TypeScript, static export | No SSR, API Routes, Server Actions, or middleware |
| Core logic | Rust crates own parsing, recurrence, interval math, scoring, reports | Keeps analysis out of UI |
| Storage | SQLite through `calguard-store` only | Local-first cache boundary |
| Sync | Rust reads local files and fetches remote ICS | Frontend only selects paths / save locations |
| Reports | Markdown and JSON renderers with privacy options | Supports default and redacted export |

## P0 Task Breakdown

1. Workspace and app shell
   - Create Rust workspace and Next.js desktop app.
   - Configure static export and Tauri v2 build paths.
   - Add dialog permissions only for open/save/message.

2. Rust core
   - Define source, event, instance, conflict, free block, score, settings,
     report, and suggestion models.
   - Implement interval overlap, merge, and subtraction.
   - Implement VEVENT parser with line unfolding, required field validation,
     line/context parse errors, status, transparency, duration, EXDATE, all-day,
     and TZID-aware date parsing.
   - Implement bounded DAILY/WEEKLY RRULE expansion with COUNT, UNTIL,
     INTERVAL, EXDATE, and a per-event max instance limit.
   - Implement conflict detection, free time calculation, focus metrics,
     overloaded-day detection, health scoring, suggestions, and report export.

3. Fixtures and tests
   - Add at least 20 ICS fixtures.
   - Cover parser, recurrence, interval, conflict, free time, scoring, and
     report privacy behavior with Rust tests.
   - Add frontend tests for empty/data dashboard, source form validation,
     conflict filtering, free-time rendering, settings, and report privacy UI.

4. Storage and sync
   - Create SQLite schema from PRD.
   - Implement source CRUD, raw ICS caching, event replacement, settings,
     ignored items, and export audit records.
   - Implement local file import limits, remote URL validation, request
     timeout, and failure behavior that preserves prior cached data.

5. Tauri command boundary
   - Implement all PRD command names.
   - Convert Rust errors to user-readable DTOs with code, message, details,
     and recoverable flag.
   - Ensure commands perform file, URL, and max-size validation in Rust.

6. Next.js UI
   - Implement onboarding, dashboard, sources, conflicts, free-time, focus,
     reports, and settings routes.
   - Provide empty states, range switching, source management, filtering,
     ignore/copy actions, privacy reminders, and settings-driven reanalysis.
   - Keep app usable in browser development with mock data while using Tauri
     commands in desktop runtime.

7. Verification and release readiness
   - Run Rust formatting/tests.
   - Run frontend lint/tests/build.
   - Configure macOS, Windows, and Linux desktop bundle targets plus CI smoke
     packaging.
   - Add release QA, draft release notes, issue templates, and README
     installation/verification guidance.
   - Verify no Vue, API Routes, SSR runtime dependency, Server Actions, or
     calendar write-back behavior exists.
   - Audit each PRD 24 checklist item with concrete evidence.

## Known MVP Limits

- RRULE support covers common DAILY/WEEKLY rules. Complex BY* combinations
  remain future work and are intentionally isolated in `recurrence.rs`.
- Timezone parsing handles UTC, floating dates/times, and common TZID names
  through `chrono-tz`; malformed timezone values produce recoverable errors.
- Cross-platform packaging is configured in GitHub Actions. Manual launch/import
  smoke checks still need to be performed on each release candidate artifact
  before publishing the draft release.
