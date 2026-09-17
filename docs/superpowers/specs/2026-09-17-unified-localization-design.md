# Unified Tracker Localization Design

## Purpose

Let a player use Mistria Tracker in the same language as their installed copy
of Fields of Mistria, without separating the Tracker interface language from
item and catalog text. The choice must be simple for new players and must not
read or modify a game save.

The initial supported set is English, Simplified Chinese, Traditional Chinese,
French, Japanese, Korean, Russian, and Spanish. It matches the official Fields
of Mistria languages available in the current game installation. The structure
must allow a later release to add another locale by adding its translation file
and registry entry rather than editing individual screens.

## Player Experience

The Settings page has one Language control:

- **Auto-detect (recommended):** read the Fields of Mistria language selected
  in Steam and use it everywhere in Tracker.
- **A named language:** use that language everywhere in Tracker until the
  player chooses Auto-detect again.

"Everywhere" means Tracker navigation, controls, settings, messages, search
labels, hints, item names, item descriptions, villager text, museum text, and
other catalog text. There are no separate interface and game-content language
controls.

If Auto-detect cannot find a valid Steam setting, the Tracker uses English. If
an installed game version has incomplete text for a selected language, Tracker
uses English only for that missing text. It never displays a missing-key token,
empty label, or damaged character encoding.

## Sources and Resolution

The selected Fields of Mistria language is stored by Steam in the game's
`appmanifest_2142790.acf` file under `UserConfig.language`. Tracker reads that
value only. It does not invoke Steam, alter Steam settings, or write to any
game file.

Tracker maintains a small mapping between Steam language identifiers and the
localization metadata shipped inside the installed `assets.zip`. The importer
enumerates translation metadata under
`assets/localization/translations/*.meta.toml`, validates each archive path and
size under the existing archive-safety rules, and parses every discovered
translation using the same item-name/description parser used for French today.
English remains the source catalog and fallback.

The user preference stores `auto` or a supported locale identifier. At startup
the app resolves that preference into one effective locale. A manual choice
wins over Auto-detect. If a manual locale is no longer present after a game
update, Tracker uses English and keeps the stored preference so the user can
change it deliberately.

## Interface Translation Architecture

All interface copy moves from component-local objects and the journal's paired
English/French arrays into `src/i18n/`:

- a locale registry containing display names, Steam identifiers, and the list
  of supported Tracker locales;
- a typed English base dictionary that defines every interface key;
- one complete dictionary per shipped locale;
- one `translate(locale, key, variables)` helper with English fallback and
  interpolation support.

Components receive the resolved locale and use the helper. They do not contain
language conditionals, paired string arrays, or private `eng | fra` types.
The registry exposes only locales whose full Tracker dictionary ships in the
app. This preserves the single-language experience: the user never selects a
language that translates item names but leaves the controls in English.

The catalog model retains localized values keyed by locale instead of a
French-only field. It resolves item text from the effective locale, then
English. Existing English and French data preserve their current display.

## Native Boundary

Rust owns all filesystem reads:

- parse the Steam app manifest from the validated Fields of Mistria library;
- enumerate and parse game translation metadata from `assets.zip`;
- expose only validated locale identifiers and localized catalog data to the
  frontend.

The frontend owns the single Language control and persists the user's choice
through the existing preferences command. It never receives arbitrary file
paths and never reads a save, Steam manifest, or ZIP directly.

## Failures and Safety

- Missing, unreadable, malformed, or unsupported app-manifest language: use
  English and continue normally.
- Malformed, unsafe, or oversized translation archive entry: reject that entry
  using the existing catalog safety behavior; do not use partial unvalidated
  text.
- Missing translation for one catalog value: use its English value.
- Missing translation for an interface key: use its English base value; this
  is an implementation guard, not a user-visible state.
- No operation in this feature writes to Fields of Mistria saves, mod files,
  Steam configuration, or game assets.

## Test Plan

Rust tests cover:

1. discovery of every valid translation metadata file in a fixture archive;
2. parsing localized item names/descriptions for Latin, Cyrillic, CJK, and
   accented text;
3. English fallback for missing translation files and missing item fields;
4. rejection of unsafe or oversized translation paths;
5. parsing Steam's `UserConfig.language` and fallback for absent or malformed
   manifests.

Frontend tests cover:

1. rendering each shipped locale without raw translation keys;
2. Auto-detect applying the effective locale to controls and catalog content;
3. manual language selection overriding Auto-detect and persisting;
4. English fallback for a missing dictionary key or catalog value;
5. Unicode text remaining visible in search results and detail panels.

Manual testing covers changing the Fields of Mistria language in Steam,
launching Tracker in Auto-detect mode, choosing a manual override, restarting
Tracker, and confirming that no game save timestamp or contents change.

## Out of Scope

- Steam-library and game-folder detection; that is a separate follow-up design.
- Installer changes, AIM compatibility changes, updates, or code signing.
- Community-created languages not included in the installed game.
- Machine-translated or incomplete Tracker UI locales. A locale is added only
  with a complete, reviewed Tracker dictionary.
