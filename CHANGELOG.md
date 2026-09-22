# Changelog

## 0.1.7 — testing

- Replaced generic undiscovered-item hints with verified area and season clues
  for fish, bugs, crops, and forageables, plus broad find areas for artifacts.
- Added recipe-acquisition clues only when confirmed by game definitions:
  store, mail, quest, museum, random reward, or starting recipe.
- Added spoiler-safe gift hints for the item's broad activity and any verified
  season or area, without revealing its identity or the villager's reaction.
- Kept untried gifts in a neutral group until tried, so their Loved/Liked
  reaction is not exposed by the journal layout.
- Suppressed misleading River clues for fish that can only come from a trap.
- Localized the new hints in all eight supported Tracker languages. Missing
  metadata omits the unsupported detail instead of guessing.
- Added a unified language setting: Auto-detect or choose one language for both
  the Tracker interface and game catalog.
- Added complete English, French, Spanish, Simplified Chinese, Traditional
  Chinese, Japanese, Korean, and Russian interface dictionaries, with
  English fallback for missing translations.
- Auto-detect recognizes all supported Fields of Mistria languages from
  Steam's language setting.
- Extracts each locale's names and descriptions from game translation metadata
  for the catalog, museum sets, and villagers; missing game text safely falls
  back to English.
- Centralized interface text so additional complete languages can be added
  without editing individual screens.

## 0.1.6 — 2026-09-22

- Reused the Tracker's tested Steam `libraryfolders.vdf` discovery in the
  Windows installer instead of maintaining a second hard-coded Steam search.
- Normalized Steam registry, library, and fallback paths to native Windows
  separators before the installer uses them.
- Validated Fields of Mistria roots with `Maybe.toml` while keeping the local
  catalog check internal; users are no longer told to locate `assets.zip`.
- Added direct, beginner-friendly AIM/MOMI instructions after the Companion is
  copied, including an optional link to MOMI's official Windows download.
- Clarified that MOMI is portable, only applies/rebuilds mods, and does not
  need to remain open while the game runs.
- Added actionable guidance when an older save version is blocked: update that
  character through the current game, then select the updated read-only save.
- Corrected the GitHub README and installation guide to match the real
  clean-machine setup flow.
- Replaced the outdated 1.0.4 manual test instructions with a 1.0.5,
  disposable-character-only checklist that never handles important saves.

## 0.1.5

- Fixed the Windows installer so the optional AIM/MOMI companion is copied to
  the detected Fields of Mistria `mods` folder with both required files.
- Fixed Steam install-path normalization and kept a validated manual game-folder
  picker for unusual Steam libraries.
- Restored passive live tracking for Fields of Mistria 1.0.5.
- Added exact active-save detection: after a room change, the companion reports
  only the active save filename and Tracker selects that exact local save slot.
- Tracker now notices when the player returns to the title menu, clears the
  active session, and shows that no save is currently loaded.
- Fixed reloading the same save without restarting Tracker. Live discoveries
  that were not saved in-game are now discarded and progress is rebuilt from
  the actual save on disk.
- Tracker now imports the active character name, farm name, and approved existing
  discoveries from a temporary read-only snapshot. The original save is never
  modified.
- Fixed the Windows application identity and shortcut icon so the Mistria
  Tracker icon is used consistently on the desktop and taskbar.
- Clarified the companion installation result: AIM or MOMI must apply/rebuild
  the installed companion once before live tracking can run.
- Approved the narrow, fail-closed save-import surface for game version 1.0.5.
- Kept **Load save file** in Settings as a manual fallback for supported saves.
- Fixed companion and portable ZIP packaging scripts and added SHA-256 release
  checksums.
