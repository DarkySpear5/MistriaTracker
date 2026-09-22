# Mistria Tracker

Mistria Tracker is an open-source, native Windows companion for Fields of Mistria.
It keeps its own tracker database and uses a passive MOMI/MMAPI companion only
to write an isolated event log. It never edits a Fields of Mistria save file.

## Installation

Download the latest Windows installer from the
[official GitHub Releases page](https://github.com/DarkySpear5/MistriaTracker/releases/latest),
then follow these steps:

1. Close Fields of Mistria and run `MistriaTracker-0.1.7-setup.exe`.
2. Leave **Live tracking companion (AIM/MOMI, recommended)** selected. The
   installer uses Steam's library list to find default and custom libraries.
   Only if automatic detection fails, choose the main *Fields of Mistria* game
   folder containing `Maybe.toml`—not `assets.zip` and not the `mods` folder.
3. The installer copies the Companion into the game's `mods` folder, but this
   alone does not activate it. Open AIM and apply/rebuild the profile, or
   download the latest Windows `ModsOfMistriaInstaller.exe` from the
   [official MOMI Releases page](https://github.com/Garethp/Mods-of-Mistria-Installer/releases/latest),
   run it, keep **Mistria Tracker Companion** checked, and click **Install**.
   MOMI is portable: it does not remain installed or run beside the game.
4. Launch Fields of Mistria normally through Steam, load your character, then
   change rooms once. Open Mistria Tracker if it is not already running.
5. The companion reports only the
   active save filename, and Tracker loads that exact character's name, farm,
   and approved existing discoveries from a temporary read-only copy. It then
   tracks new discoveries automatically while you play. If automatic detection
   is unavailable, Settings > **Load save file** remains a manual fallback.

Run AIM's apply/rebuild action or MOMI's **Install** button again after every
Fields of Mistria update and after replacing the Companion files.

Save import is intentionally fail-closed: if the selected game version is not
approved yet, Tracker reports that clearly and leaves both the save and tracker
database unchanged.

Tracker finds Fields of Mistria automatically in normal and custom Steam
libraries. If it cannot, open Settings and choose the main game folder
containing `Maybe.toml`. Tracker reads its local catalog but does not change
game files or saves.

Tracker language defaults to **Auto-detect**, matching the language selected for
Fields of Mistria in Steam. English, French, Spanish, Simplified Chinese,
Traditional Chinese, Japanese, Korean, and Russian are available for the
interface; supported game text follows the same selection. Missing translations
safely fall back to English. You can choose a different language in Settings,
and that single choice applies to both.

AIM or MOMI is needed to apply the Companion used for automatic active-save
detection and live tracking. Neither tool needs to remain open while playing.
Without an applied Companion, Mistria Tracker can still import a supported save
through Settings > **Load save file** and show its local tracker data. The
tracker never edits a Fields of Mistria save file.
See the [complete MOMI installation and troubleshooting guide](docs/MOMI_INSTALL_GUIDE.md)
if nothing appears.

## Code signing policy

Official release artifacts are built from the public `main` branch, tested, and
manually approved by the project maintainer before signing and publication.
The maintainer (`DarkySpear5`) is the committer, reviewer, and release approver
for this solo-maintained project. The program does not transfer information to
other networked systems unless the user explicitly requests it.

Current releases are unsigned. Windows may show a reputation warning for a new
release; only download official assets from this repository and verify the
published SHA-256 checksum. See the full [code signing policy](CODE_SIGNING_POLICY.md)
and [privacy policy](PRIVACY.md).

## Safety contract

- The tracker database, preferences, and notes are app-owned local data.
- The companion observes documented MMAPI hooks and returns `undefined` from
  its only filter, so it does not replace a game value or change gameplay.
- The companion writes only its own log under
  `%LOCALAPPDATA%\FieldsOfMistria\mod_data\mistria_tracker_companion\logs\`.
- The desktop app refuses to begin live tracking unless a game build has passed
  an explicit compatibility approval. Unknown or probe-only builds remain
  paused.
- Spoiler-free is the default. Item and gift identities are shown only after
  their own observed event; aggregate counts may remain visible.

## Local development

Before starting new work on `testing`, fetch the remote repository and
fast-forward `testing` to the active `origin/main`. Do not build testing work
on an older public release.

```powershell
git fetch origin
git switch testing
git merge --ff-only origin/main
pnpm test -- --run
pnpm test:companion
pnpm typecheck
cargo test --offline --manifest-path src-tauri/Cargo.toml
pnpm tauri build --no-bundle
```

The release executable is written to
`src-tauri\target\release\mistria-tracker.exe`.

## Companion preflight

The companion is intentionally an unpacked folder at
`companion\mistria_tracker_companion`; MOMI does not install zip files. Before
copying it into a game `mods` folder, run MOMI's no-write preflight with MOMI
0.14.1 or newer and the .NET 10 SDK:

```powershell
dotnet run --project ModsOfMistriaCommandLine -- --lint `
  "<tracker>\companion\mistria_tracker_companion" `
  "<Fields of Mistria>\assets.zip" `
  --strict-lints --compile-check require
```

Do not install a companion that MOMI reports as skipped. MOMI validates the
entire mod before it writes game scripts.

## Disposable-profile live validation

Until live validation is complete, use only a newly created disposable character:

1. Do not open, list, copy, read, or select an important save.
2. Run the preflight above. A failed or unavailable preflight means no install.
3. Put the unpacked companion folder directly under the game `mods` folder and
   run MOMI only after the preflight passes.
4. Launch Fields of Mistria, select the disposable character yourself, and obtain or gift one known
   item.
5. Verify only the companion log. Do not inspect any game save file.
6. Confirm the tracker recorded the event only after the desktop readiness gate
   approves the installed game build.

The eventual tracker supports any profile through its numeric tracker profile
identifier, but live testing remains limited to a disposable profile until this checklist passes.

## Source review package

This repository contains the complete companion and desktop-app source for
independent review. It intentionally excludes
game saves, save backups, local diagnostics, dependency caches, and generated
installer binaries. The installer recipe is included in `tools/installer.nsi`.

The app is read-only with respect to Fields of Mistria saves: it observes the
running game through the isolated companion log and stores tracker discoveries
in its own application data. A second tracker instance is rejected, and live
tracking is gated by compatibility approval.

## Rebuilding from source

Install Node.js, pnpm, Rust, and the Tauri prerequisites, then run:

```powershell
pnpm install
pnpm test -- --run
pnpm test:companion
pnpm typecheck
cargo test --offline --manifest-path src-tauri/Cargo.toml
pnpm tauri build --no-bundle
```

The Windows installer script is `tools/installer.nsi`; it consumes the release
executable produced by Tauri. No signing certificate or credentials are stored
in this repository.

Maintainers can build a portable ZIP and its checksum with
`pnpm package:portable`. It is a developer distribution option and is not part
of the official Windows installer release.

End-user setup is documented in [`docs/MOMI_INSTALL_GUIDE.md`](docs/MOMI_INSTALL_GUIDE.md).
