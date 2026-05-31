# CalGuard Release QA Checklist

Use this checklist before cutting an MVP release tag. The automated CI workflow
builds on macOS, Windows, and Linux; this checklist covers the user-facing
desktop paths from PRD section 19.4.

## Required Platforms

| Platform | Bundle | Manual Smoke |
| --- | --- | --- |
| macOS | `CalGuard_*.dmg` | Install DMG, launch app, import `fixtures/simple.ics` |
| Windows | `CalGuard_*_x64-setup.exe` | Install NSIS package, launch app, import `fixtures/simple.ics` |
| Linux | `calguard_*.deb` | Install deb package, launch app, import `fixtures/simple.ics` |

## Manual End-to-End Cases

1. Launch without an account or network login prompt.
2. Import `fixtures/simple.ics` as a local calendar source.
3. Add a temporary HTTP ICS URL and confirm the source syncs successfully.
4. Add an invalid remote URL and confirm the app shows a recoverable error.
5. Delete a calendar source and confirm Dashboard metrics refresh.
6. Clear local cache, re-import `fixtures/simple.ics`, and confirm analysis returns.
7. Export Markdown and open the file in a text editor.
8. Export JSON and confirm `schemaVersion` exists at the top level.
9. Enable privacy export and confirm event title, location, and description are redacted.

## Local Verification Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --filter @calguard/desktop lint
pnpm --filter @calguard/desktop test
pnpm --filter @calguard/desktop build
pnpm --filter @calguard/desktop exec tauri build --bundles dmg --ci --no-sign
```

## Release Gate

A release tag is eligible only after:

- `CI` passes on `main`.
- `Release Bundles` produces macOS DMG, Windows NSIS, and Linux deb artifacts.
- The platform manual smoke table above is completed for the release candidate.
