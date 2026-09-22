# Spoiler-safe Hints Implementation Plan

> **For agentic workers:** Implement these tasks in order on the existing `testing` worktree. Each task ends with a focused verification and a small commit.

**Goal:** Give undiscovered fish, bugs, crops, forageables, artifacts, recipes, and gifts useful hints based on verified game metadata without revealing identities.

**Architecture:** Extend the bounded journal catalog extraction with typed availability and acquisition facts. The Rust snapshot emits only safe codes; one frontend hint dialog translates and presents those codes using the existing unified language choice.

**Tech Stack:** Rust, `toml`, `serde_json`, Tauri 2, React, TypeScript, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-22-spoiler-safe-hints-design.md`

## Global constraints

- Work only in `.worktrees/tracker-testing` on `testing`; `origin/main` and `testing` both start at `02f30b1`.
- Preserve the existing uncommitted eight-language implementation as a separate baseline commit. Never stage pre-existing `tools/installer/` artifacts.
- Treat `assets.zip` as read-only, cap newly admitted definition files at the existing 16 MiB per-entry limit, and never open or modify a save file.
- Hidden item names, IDs, descriptions, exact locations, source links, images, and untried gift reactions stay private.
- Add no updater, installer build, branch merge, or public release.

## Review focus

1. A mine-only fish with the game's default `water_type = river` must show mines, never river. Task 2 tests this.
2. An item with no validated season or area must omit those chips rather than guess. Task 3 tests this.
3. A recipe available by shop and mail must consistently use the shop clue. Task 2 tests this.
4. A hidden gift must not disclose its item ID, name, or reaction anywhere in the snapshot. Task 3 tests this.
5. All eight locales must display the same semantic hint without a raw key or undiscovered identity. Task 4 tests this.

---

### Task 1: Freeze the completed language baseline

**Files:** Existing tracked localization edits and `src/i18n/{es,chs,cht,ja,ko,ru}.ts` only.

**Interfaces:** Produces the eight-locale `translate(locale, key, variables)` function consumed by Task 4.

- [ ] Check `git diff --check`, the current branch, and the explicit staged file list. `tools/installer/` remains untracked.
- [ ] Run `pnpm build` and `cargo check --manifest-path src-tauri/Cargo.toml --offline` to verify the existing localization baseline.
- [ ] Stage only the localization, version, and related documentation files visible in the pre-task diff; commit `feat: complete eight-language tracker foundation`.

### Task 2: Extract verified hint facts from game definitions

**Files:** Create `src-tauri/src/journal/hints.rs`; modify `src-tauri/src/journal/mod.rs`, `catalog.rs`, and `tests.rs`.

**Interfaces:** `HintFacts { areas: Vec<HintArea>, seasons: Vec<HintSeason>, recipe_source: Option<RecipeSource> }` is internal. `populate_hint_facts(catalog: &mut JournalCatalog, documents: &BTreeMap<String, toml::Value>)` uses only whitelisted TOML documents. Task 3 consumes `Entry.hint_facts`.

- [ ] Add focused failing tests for fish retrieval, bug tag/season, crop and forage season, artifact group area, shop/mail/random recipe source, and shop priority. Use tiny parsed TOML values, not a user's save.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml journal::tests::hint --offline`; confirm each test fails because the facts are absent.
- [ ] Implement typed parsers. Accept a season only from `spring|summer|fall|winter`, an area only from a finite game-area map, and recipe sources only from explicit recipe references. Interpret scalar and list forms; use the game's `default` table only where that table defines the field. A `retrieval = mines` fish takes precedence over default water type.
- [ ] Admit only the required extra TOML paths in `JournalCatalog::extract`, respecting the existing entry-size check, then invoke `populate_hint_facts` after entries and museum sets are built.
- [ ] Run the focused Rust tests and `cargo fmt --manifest-path src-tauri/Cargo.toml --check`; commit `feat: derive hint facts from verified game metadata`.

### Task 3: Publish only safe hint codes

**Files:** Modify `src-tauri/src/journal/hints.rs`, `view.rs`, and `tests.rs`.

**Interfaces:** `pub fn hint_for(entry: &Entry, gift: bool) -> Hint` returns a serializable `Hint { kind, activity, areas, seasons, source }`. Task 4 consumes the snapshot's `entry.hint` and `gift.hint` objects.

- [ ] Add failing snapshot tests: fish yields area and season without identity, missing metadata yields no fabricated facts, and a hidden gift yields activity without ID/name/reaction.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml journal::tests::hint --offline`; confirm expected failures.
- [ ] Replace the old string `hint_for` in `view.rs` with typed safe hints. Add hints to hidden gift slots. Keep all existing reveal rules unchanged.
- [ ] Run the focused Rust tests and `cargo fmt --manifest-path src-tauri/Cargo.toml --check`; commit `feat: expose spoiler-safe hints in journal snapshots`.

### Task 4: Render localized category hints

**Files:** Create `src/journal/HintCard.tsx` and `HintCard.test.tsx`; modify `src/journal/types.ts`, `screens.tsx`, `src/app/DesktopApp.tsx`, and all eight `src/i18n/*.ts` dictionary files.

**Interfaces:** `Hint` in `types.ts` mirrors Rust `Hint`; `HintCard({ hint, language, onClose })` renders the translated intro, area/season chips, and recipe-source line.

- [ ] Add failing UI tests for `Pond · Summer`, shop recipe copy, an unknown-data fallback, and translated output in all eight locale dictionaries; assert no raw `hint_` key or hidden name appears.
- [ ] Run `pnpm exec vitest run src/journal/HintCard.test.tsx`; confirm expected failures.
- [ ] Implement the typed hint callback and modal. Reuse existing season/area translation keys where present; add the missing area, activity, and recipe-source keys in all eight dictionaries. Remove the hard-coded old-hint whitelist.
- [ ] Run the focused UI tests and `pnpm build`; commit `feat: show localized spoiler-safe discovery hints`.

### Task 5: Document and verify the testing update

**Files:** Modify `docs/ROADMAP.md`, `CHANGELOG.md`, and `README.md` only as needed for the new hint behavior.

**Interfaces:** No new runtime interface.

- [ ] Update the 0.1.7 testing notes to describe which clues are data-backed and where generic fallback remains.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml --offline`, `pnpm test -- --run`, `pnpm build`, and `git diff --check`. Read exit codes and resolve failures attributable to this change.
- [ ] Inspect a read-only journal extraction against the installed 1.0.5 archive; confirm representative fish, bug, crop, forage, artifact, and recipe hint facts without opening a save.
- [ ] Commit `docs: describe 0.1.7 hint coverage and fallbacks`; report the testing branch commit IDs and any manual-review limits.
