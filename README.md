# Mistria Tracker

Mistria Tracker is an open-source, native Windows companion for Fields of Mistria.
It keeps its own tracker database and uses a passive MOMI/MMAPI companion only
to write an isolated event log. It never edits a Fields of Mistria save file.

## Installation

The desktop tracker and the game companion are separate:

1. To track new discoveries while playing, install AIM or MOMI/Mods of Mistria
   with compatible MMAPI hooks first.
2. Run `MistriaTracker-0.1.4-setup.exe` and install it normally. Tick
   **Add a desktop shortcut** if you want one.
3. Leave **Live tracking companion (AIM/MOMI, recommended)** selected to
   install live tracking. Setup finds normal Steam installs automatically. On
   an unusual Steam location, choose the *Fields of Mistria game folder* (the
   folder containing `assets.zip`), not its `mods` folder.
4. Start Fields of Mistria through AIM or MOMI, then open Mistria Tracker.
5. The tracker automatically loads existing discoveries when it opens. With
   AIM/MOMI and the companion installed, it also begins tracking new
   discoveries automatically while you play. After loading a save for the
   first time, change rooms once to start the companion log.

Tracker finds Fields of Mistria automatically in normal Steam libraries. If it
cannot, open Settings and choose the game folder that contains `assets.zip`.
Tracker does not change game files or saves.

MOMI is only needed for live tracking. Without it, Mistria Tracker can still
show discoveries already found from its local tracker data. The tracker never
edits a Fields of Mistria save file.
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

```powershell
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
of the official 0.1.4 release.

End-user setup is documented in [`docs/MOMI_INSTALL_GUIDE.md`](docs/MOMI_INSTALL_GUIDE.md).
