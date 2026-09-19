# Mistria Tracker Roadmap

This roadmap tracks work after the 0.1.4 release candidate. Every stage is
read-only with respect to Fields of Mistria saves.

## 1. Security hardening — current priority

Status: the core read limits and installer deletion fix are included in 0.1.4.

- Bound tracker-owned backup, vault, snapshot, and companion-log reads so a
  malformed or unexpectedly large local input cannot exhaust memory.
- Preserve the existing restrictions: no save writes, process injection,
  gameplay changes, arbitrary command execution, or background network access.
- Verify Rust tests and perform a focused review before committing this work.

## 2. Reliable live tracking and game discovery — next

Status: 1.0.5 live-event compatibility, active-profile preservation, and
validated Steam discovery are included in 0.1.4. Manual game testing remains
the next reliability checkpoint.

- Replace the hard-coded default Steam path with validated Steam-library
  discovery, bounded drive-root fallback, and one manual folder choice.
- Keep the Tracker paused rather than guessing when the game cannot be found.
- Make active-save detection robust: use only the current companion event log,
  handle a fresh profile after a room change, and prevent two installed
  Tracker-companion versions from competing for the same MMAPI identifier.
- Test default Steam, a secondary drive, stale paths, missing logs, and a
  profile switch without opening or changing a game save.

## 3. Unified language support

Status: design already approved; implementation begins after stage 2 supplies
the validated Steam library boundary.

- One language choice only: **Auto-detect** or one manual language.
- The choice controls both Tracker controls and extracted game catalog text.
- Start with English, Simplified Chinese, Traditional Chinese, French,
  Japanese, Korean, Russian, and Spanish, each with safe English fallback.
- Make the translation structure extensible without exposing unfinished
  languages to players.

## 4. GitHub update delivery

Status: requires a separate security and release design.

- Check GitHub releases only when the player explicitly enables update checks.
- Download and install only a verified release artifact; never auto-run an
  unverified download.
- Keep the current installed version usable if a check, download, or update
  fails.
- Do not turn this into a subscription or cloud-account requirement.

## 5. Useful, spoiler-safe hints

Status: future product design after the reliability work.

- Replace generic hints with category-specific facts from validated game data.
- Example: an undiscovered fish can show a non-spoiling season and area such
  as `Pond · Summer`, rather than its name or exact spawn point.
- Adapt the same principle to bugs, crops, artifacts, recipes, gifts, and
  other categories only after their reliable source data is verified.

## 6. Beginner-friendly installer

Status: the first safe automatic Steam-location check, manual fallback, and
desktop-shortcut choice are included in 0.1.4. Broader installer improvements
stay on the roadmap.

- Detect the validated game/mods folder automatically and offer the companion
  as a clearly labelled recommended live-tracking component.
- Keep one safe manual fallback only when detection fails.
- Add an optional **Add a desktop shortcut** checkbox.
- Preserve Tracker data on normal uninstall and explain that choice clearly;
  consider a separate, explicit data-removal option later.
- Produce one new installer after the finished app is tested, rather than
  repeatedly rebuilding installers while core behavior is changing.

## Release rule

No stage is merged into a public installer until its focused tests pass, a
manual disposable-save check succeeds, and the resulting installer is built
and scanned as that exact release artifact. Microsoft/Nexus/VirusTotal review
status for an older installer does not automatically approve a new build.
