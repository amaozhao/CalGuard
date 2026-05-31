# CalGuard MVP

This draft release contains the CalGuard local-first desktop MVP.

## Highlights

- Import local `.ics` files and remote ICS URLs.
- Analyze upcoming calendar health for 7, 14, or 30 days.
- Detect conflicts, overloaded days, fragmented schedules, and focus blocks.
- Export Markdown or JSON reports with optional privacy redaction.
- Store data locally in SQLite without account registration or calendar write-back.

## Verification

- Rust formatting, clippy, and workspace tests.
- Frontend typecheck, Vitest tests, and Next.js static export.
- macOS DMG, Windows NSIS, and Linux deb bundle jobs.

## Release QA

Before publishing this draft, complete `docs/RELEASE_QA.md` on macOS, Windows,
and Linux using the attached artifacts.
