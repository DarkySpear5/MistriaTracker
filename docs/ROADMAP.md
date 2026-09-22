# Mistria Tracker Roadmap

This roadmap tracks work after the 0.1.5 reliability release. Every stage is
read-only with respect to Fields of Mistria saves.

## 1. Security hardening — current priority

Status: the 0.1.5 hardening is published. The testing branch is synchronized
from the active main release before every new stage and receives a full test,
build, and security scan before it is pushed.

- Bound tracker-owned vault, snapshot, and companion-log reads so a
  malformed or unexpectedly large local input cannot exhaust memory.
- Preserve the existing restrictions: no save writes, process injection,
  gameplay changes, arbitrary command execution, or background network access.
- Verify Rust tests and perform a focused review before committing this work.

## 2. Reliable save import and live tracking — current

Status: 0.1.5 removes automatic save-path guessing, adds an explicit read-only
`.sav` picker, and approves the narrow 1.0.5 import surface. Live tracking
remains event-only through the Companion log. Older save versions remain
blocked until separately verified.

- Keep validated Steam-library discovery, bounded drive-root fallback, and one
  manual folder choice.
- Keep the Tracker paused rather than guessing when the game cannot be found.
- Make active-profile detection robust: use only the current companion event log,
  handle a fresh profile after a room change, and prevent two installed
  Tracker-companion versions from competing for the same MMAPI identifier.
- Let the user choose a save from any accessible drive without opening or
  changing the original file; parse only a bounded tracker-owned snapshot.
- Test default Steam, a secondary drive, stale paths, missing logs, and a
  profile switch without opening or changing a game save.

## 3. Unified language support

Status: 0.1.7 testing implementation adds complete interface dictionaries and
game-catalog extraction for all eight planned languages. It remains on the
testing branch pending language review.

- One language choice only: **Auto-detect** or one manual language.
- The choice controls both Tracker controls and extracted game catalog text.
- Auto-detect the installed game language or let the player choose one language
  for both Tracker UI and game-content text: English, Simplified Chinese,
  Traditional Chinese, French, Japanese, Korean, Russian, or Spanish.
- Keep English fallback for missing game/catalog translations and do not expose
  incomplete interface locales.
- Add additional complete official game locales in later testing stages.

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

Status: the testing branch now reuses the app's Steam `libraryfolders.vdf`
discovery, validates the game root with `Maybe.toml`, verifies both Companion
files, and explains the required AIM/MOMI apply step. Linux/SteamOS packaging
and a separately audited modern installer folder-picker dependency remain
separate work.

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
