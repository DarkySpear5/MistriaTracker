# Changelog

## 0.1.5

- Fixed the Windows installer so the optional AIM/MOMI companion is copied to
  the detected Fields of Mistria `mods` folder with both required files.
- Fixed Steam install-path normalization and kept a validated manual game-folder
  picker for unusual Steam libraries.
- Restored passive live tracking for Fields of Mistria 1.0.5.
- Added exact active-save detection: after a room change, the companion reports
  only the active save filename and Tracker selects that exact local save slot.
- Tracker now imports the active character name, farm name, and approved existing
  discoveries from a temporary read-only snapshot. The original save is never
  modified.
- Approved the narrow, fail-closed save-import surface for game version 1.0.5.
- Kept **Load save file** in Settings as a manual fallback for supported saves.
- Fixed companion and portable ZIP packaging scripts and added SHA-256 release
  checksums.

