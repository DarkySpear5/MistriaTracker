# Hint implementation context pack

## Branch and preservation

Use the linked worktree `.worktrees/tracker-testing` on `testing`. At planning
time, `testing`, `origin/testing`, and public `origin/main` all resolve to
`02f30b1`. The linked worktree contains uncommitted 0.1.7 language work; keep
it intact and commit it separately before hint code. `tools/installer/` is
pre-existing untracked output and must not be staged. The local branch called
`master` is an unrelated documentation-only history; do not merge it into this
app branch. No push, merge, updater, installer build, or release is part of
this work.

## Existing flow

- `src-tauri/src/journal/catalog.rs` extracts entries from bounded TOML in
  `assets.zip`; `Entry` holds seasons, places, recipe links, and translations.
- `src-tauri/src/journal/view.rs` applies the spoiler boundary and currently
  emits a generic string `hint` for each hidden entry. Hidden gift slots use
  the same generic `gift` string in the frontend.
- `src/journal/screens.tsx` calls `onHint` when a hidden slot is clicked;
  `src/app/DesktopApp.tsx` renders the modal. `src/i18n/registry.ts` has eight
  locale dictionaries and one unified language selection.
- `src-tauri/src/journal/tests.rs` already verifies that hidden names and
  detailed locations do not appear in the snapshot.

## Verified installed-game definition shapes

- `fish.toml`: `seasons` may be false or an array; `water_type` and `retrieval`
  may be scalars or arrays; `locations` can identify the deep woods. Mines
  retrieval must outrank default river water type.
- `bugs.toml`: `default.seasons` lists all seasons; per-bug `tag` can indicate
  beach, deep woods, or mines; `dungeon_biome` identifies a mine region.
- `object_prototypes/crop.toml`: `seasons` is a string, array, or default -1.
  `forageables.toml` groups known forage IDs by season and identifies sand
  forageables.
- `artifacts.toml`: `[locations]` maps broad areas to artifact set groups.
  `museum_wings/archaeology.toml` gives the member IDs for those groups.
- `stores.toml`: stock contains `{ recipe_scroll = "..." }`,
  `{ crafting_scroll = "..." }`, and `include_recipe = true` item entries.
  `letters.toml`, quests, museum rewards, festivals, wishing-well, and
  chicken-statue definitions contain additional explicit recipe references.
  Recipe items may declare `recipe_is_default = true`.

## Core safety rule

Only finite, translated codes cross the Rust-to-UI hint boundary. Raw game
names, IDs, paths, prose descriptions, and exact spawn data are not hint
payloads. Missing or conflicting evidence falls back to a general clue.
