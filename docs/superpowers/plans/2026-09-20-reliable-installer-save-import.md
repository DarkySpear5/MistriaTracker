# Reliable installer and explicit save import — implementation plan

> **For implementation:** use the TDD workflow: make each test fail for the old behavior, implement the smallest change, then run the focused test and the full supported suite.

**Goal:** deliver the Windows 0.1.5 repair without modifying the published 0.1.4 release. The optional companion must install with verified files, and the desktop tracker must import one user-selected `.sav` through a read-only tracker-owned snapshot.

**Architecture:** keep passive live tracking event-only; remove Desktop-backup discovery and hard-coded save polling. Add a native file-picker bridge whose backend validates and snapshots the selected file before parsing. Keep the existing transactional tracker database and bounded snapshot storage. Make the NSIS optional companion section fail safely and report a clear message instead of exposing retry prompts.

**Testing:** Rust unit/integration tests for selected-file validation, snapshot/import cleanup, unsupported versions, and event-only live tracking; Vitest tests for the Settings picker; static installer tests for required companion payload and post-copy verification; CI on Node 24 and the Windows installer workflow.

## Task 1: Establish the 0.1.5 test seams

**Files:** `src-tauri/src/save/backup.rs`, `src-tauri/src/safety/snapshot.rs`, `src-tauri/src/commands.rs`, `src-tauri/tests/commands.rs`, `src/features/settings/SettingsPage.test.tsx`, `src/lib/nativeTracker.test.ts`, `tools/companion-safety.test.ts`.

1. Add failing Rust tests proving a selected `.sav` outside the game saves directory is accepted only as a regular file, folders/non-`.sav` paths are rejected, oversized inputs are rejected, and the temporary tracker snapshot is removed after success or parse failure.
2. Add a failing Rust command test for an unsupported parser version that returns a non-import result without inventing a profile or writing tracker discoveries.
3. Add a failing frontend test that Settings exposes `Load save file`, invokes the file picker callback, and no longer renders `Mistria Save Backups` copy.
4. Add a failing native bridge test for the selected-save command and installer static assertions for the two required companion files plus post-copy checks.

## Task 2: Replace automatic backup/save polling with explicit read-only import

**Files:** `src-tauri/src/safety/paths.rs`, `src-tauri/src/safety/snapshot.rs`, `src-tauri/src/save/backup.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, `src/lib/nativeTracker.ts`, `src/app/DesktopApp.tsx`, `src/features/settings/SettingsPage.tsx`, `src/journal/copy.ts`.

1. Add a validated selected-save path type (canonical regular `.sav`, bounded size) without weakening the existing live-save path boundary.
2. Generalize the bounded snapshot operation to accept that validated path, preserving source fingerprints and tracker-local temporary storage.
3. Add `import_selected_save` that receives only the user-selected path, snapshots it, derives the profile only from the validated filename, checks the compatibility parser, parses the snapshot, imports transactionally, stores journal evidence, and always cleans the snapshot and manifest.
4. Return explicit result states for imported, already imported, invalid/unsupported save, and unsupported game version. Never mutate or scan the original file.
5. Add a Tauri command and TypeScript wrapper using the native file dialog (`.sav` filter). Remove `import_latest_desktop_backup`, Desktop/OneDrive path guesses, `reconcile_active_live_save`, hard-coded `%LOCALAPPDATA%` save lookup, startup import, and polling reconciliation. Keep companion event polling and profile switching.
6. Update Settings copy and component props so the single action is beginner-friendly: `Load save file`; explain that the selected save is read-only and copied temporarily; display the exact result message.

## Task 3: Make the companion install safe and verifiable

**Files:** `tools/installer.nsi`, `tools/companion-safety.test.ts`, `companion/mistria_tracker_companion/manifest.json`, `companion/mistria_tracker_companion/gml/MistriaTrackerCompanion.gml`, `.github/workflows/build-windows.yml`, `tools/package-companion.ps1`, `tools/build-portable.ps1`.

1. Keep automatic game-folder detection only when `assets.zip` proves the folder; otherwise use the folder picker.
2. Preflight the exact `mods\MistriaTrackerCompanion` destination, create it, copy only the two required payload files, and verify both files exist.
3. Do not remove unrelated mod files or recursively delete an existing companion folder. Catch expected failures and finish the tracker installation with one clear explanatory message; never expose Abort/Retry/Ignore for this path.
4. Add build-time checks that fail before artifact upload if either payload file is missing, and update all 0.1.5 artifact/version paths.

## Task 4: Remove obsolete code and update documentation

**Files:** `src-tauri/tests/commands.rs`, `src-tauri/tests/installed_save_probe.rs`, `src/app/App.test.tsx`, `README.md`, `docs/MOMI_INSTALL_GUIDE.md`, `docs/ROADMAP.md`, related copy/tests.

1. Delete tests and fixtures whose only purpose is Desktop-backup import or automatic save discovery; retain database rollback-backup tests and companion event tests.
2. Remove stale copy and documentation that promises `Mistria Save Backups` or a Desktop path. Document the explicit save picker, read-only behavior, supported-version status, and the companion’s first room-change requirement.
3. Record Linux/SteamOS as a follow-up rather than implying the NSIS build supports it.

## Task 5: Verify and package 0.1.5

1. Run focused Rust/Vitest/installer tests, then the complete Rust and Node 24 suites through CI.
2. Build the Windows installer and inspect its contents for both companion files and the new version.
3. Manually test default Steam, secondary Steam library/manual game folder, companion write failure, selected save import, unsupported save version, and one live companion event.
4. Only after all evidence passes, prepare the 0.1.5 release and checksum. Do not replace or mutate 0.1.4 assets.
