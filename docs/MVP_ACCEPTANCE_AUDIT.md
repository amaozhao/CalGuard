# CalGuard MVP Acceptance Audit

Audit date: 2026-06-01

## Evidence Collected

| Gate | Evidence |
| --- | --- |
| Deterministic app icon | `node scripts/generate-icon.mjs` passed; `tauri icon` generated PNG, ICNS, and ICO assets |
| Rust formatting | `cargo fmt --all --check` passed |
| Rust static analysis | `cargo clippy --workspace --all-targets -- -D warnings` passed |
| Rust tests | `cargo test --workspace` passed: 22 tests across core, fixtures, store, and sync |
| Frontend typecheck | `pnpm --filter @calguard/desktop lint` passed (`tsc --noEmit`) |
| Frontend tests | `pnpm --filter @calguard/desktop test` passed: 9 tests |
| Next static export build | `pnpm --filter @calguard/desktop build` passed; all app routes are static |
| Workflow YAML | Ruby YAML parse passed for CI, release, and issue template files |
| Static screenshot | `docs/screenshots/dashboard.png` captured from `apps/desktop/out` at 1440x1000 |
| Tauri macOS DMG | `pnpm --filter @calguard/desktop exec tauri build --bundles dmg --verbose --ci --no-sign` passed |
| DMG integrity | `hdiutil verify target/release/bundle/dmg/CalGuard_0.1.0_aarch64.dmg` passed |

## Product Checklist

| Requirement | Status | Evidence |
| --- | --- | --- |
| No registration required | Complete | No auth/account code or route exists |
| Import local `.ics` file | Complete | `add_local_ics_source` command + Onboarding/Sources UI + validation tests |
| Add remote `.ics` URL | Complete | `add_remote_ics_source` command + Onboarding/Sources UI |
| Set working hours | Complete | Settings and onboarding write `AnalysisSettings` |
| Select 7 / 14 / 30 day range | Complete | Dashboard/onboarding/settings controls |
| Dashboard | Complete | `/dashboard` static route and `DashboardView` |
| Conflict list | Complete | `/conflicts` route with severity filter and ignore/copy actions |
| Free time list | Complete | `/free-time` route with focus threshold control and display test |
| Health score | Complete | Rust `score_health()` with positive/negative reasons |
| Deduction reasons | Complete | `ScoreReason` DTOs and Dashboard Top Risks |
| Markdown export | Complete | `render_markdown_report()` and `export_report` |
| JSON export | Complete | `render_json_report()` includes schema version |
| Privacy export | Complete | JSON and Markdown hide event title/location/description when configured |
| Clear local cache | Complete | `clear_cache` command + Settings UI + store regression test |

## Technical Checklist

| Requirement | Status | Evidence |
| --- | --- | --- |
| Next.js `output: 'export'` | Complete | `apps/desktop/next.config.mjs` |
| Tauri `frontendDist` points to `../out` | Complete | `apps/desktop/src-tauri/tauri.conf.json` |
| No Vue | Complete | No Vue dependencies/files |
| No API Routes | Complete | No `route.ts` files |
| No SSR runtime dependency | Complete | Static export build passes |
| No Server Actions | Complete | No `"use server"` or server actions |
| Core analysis in Rust | Complete | `crates/calguard-core` owns parser/recurrence/analysis/scoring/report |
| SQLite local storage | Complete | `crates/calguard-store` schema/repository |
| Unit tests pass | Complete | `cargo test --workspace` |
| Fixture tests pass | Complete | 20 ICS fixtures + integration tests |
| Packaged app builds | Complete | macOS DMG generated and verified locally |
| Cross-platform bundle config | Complete | Platform configs and CI matrix for macOS DMG, Windows NSIS, Linux deb |
| Release prep assets | Complete | README, screenshot, release notes template, QA checklist, issue templates |

## Security and Privacy Checklist

| Requirement | Status | Evidence |
| --- | --- | --- |
| Default no upload | Complete | No cloud API/account path; remote fetch only user-provided URL |
| Default no calendar write-back | Complete | No create/edit/delete calendar event commands |
| Remote ICS timeout | Complete | `REMOTE_TIMEOUT_SECONDS = 10` |
| ICS file size limit | Complete | `MAX_ICS_BYTES = 20 MB` |
| Remote URL validation | Complete | HTTP/HTTPS enforcement and remote sync tests |
| Remote failure safety | Complete | Failed sync marks source failed without replacing cached events |
| Export privacy reminder | Complete | Reports page notice before save |
| Privacy mode hides title/location/description | Complete | Core report tests cover title/location/description redaction |
| Clear cache | Complete | Repository `clear_cache()` deletes raw calendars and events |

## Release Gates

| Gate | State |
| --- | --- |
| Windows/Linux build verification | Configured in `.github/workflows/ci.yml`; will run on GitHub after push |
| Release artifact generation | Configured in `.github/workflows/release.yml` for tag-based draft releases |
| Windows/Linux manual launch/import smoke | Documented in `docs/RELEASE_QA.md`; must be completed on release candidate artifacts before publishing |
| Code signing/notarization | Not included in MVP; local package verification uses `--no-sign` |
| Advanced RRULE coverage | P0 supports DAILY/WEEKLY with COUNT, UNTIL, INTERVAL, EXDATE; complex BY* rules remain isolated future work |
