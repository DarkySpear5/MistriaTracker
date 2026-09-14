# Desktop Journal Implementation Plan

> Execute in the existing isolated build worktree. User authorized the design choices and continued implementation. No further design-permission round is required.

**Goal:** Replace the flat tracker UI with a coherent native desktop journal using actual game catalog and verified discoveries.

**Architecture:** Preserve the current companion and persistence interfaces. Add a journal registry and spoiler-filtered snapshot to the Rust core; consume it in focused React screens sharing toolbar, art cache, and item drawer components.

**Tech Stack:** Tauri 2, Rust, SQLite, React 19, TypeScript, local game PNG assets.

**Spec:** ../specs/2026-09-09-desktop-journal-design.md

## Global constraints

- No writes to game assets, companion, or any original or backup game save.
- English/French, no external services or runtime font requests.
- Unknown identities never enter search or related links in spoiler-free mode.
- Learned recipes and acquired cooked dishes remain independent.
- Preserve local art pack performance and existing tracker persistence.

## Task 1: Canonical catalog and evidence

Files: create `src-tauri/src/journal/{mod,catalog,evidence,view}.rs`; integrate in `app_state.rs`, `commands.rs`, `lib.rs` and `save/backup.rs`.

Interfaces: `JournalCatalog::extract(&AssetsZip)`, `JournalEvidence::from_vault(&Vault)`, `journal_snapshot(&TrackerState)`; JSON snapshot contains entries, museum wings/sets, villagers and category progress. Hidden entries contain opaque slot keys and no identity-bearing fields.

- [ ] Inspect installed TOML and approved backup field shapes with read-only tools.
- [ ] Parse explicit categories, set membership, NPC gifts, recipes, and structured availability; reject malformed recognized fields.
- [ ] Persist profile-specific additional evidence in tracker settings; replay must be idempotent and must not remove live discoveries.
- [ ] Regression checks: a known recipe never discovers its output item; a found item never counts as donated; an unmet NPC and unknown gift remain absent from serialized identity fields; existing backup file hash remains unchanged.

## Task 2: Desktop visual system and navigation

Files: `src/app/App.tsx`, `src/app/app.css`, `src/journal/{types,copy,components,search}.tsx` and screen modules.

- [ ] Build fixed sidebar/search toolbar and coherent dusk design tokens. Keep native window controls and minimum-size behavior.
- [ ] Build category launcher, four museum columns, portrait cards and data dashboard with actual snapshot values.
- [ ] Implement contextual search result selection, adjacent filter/sort menus and side drawer navigation.
- [ ] Test substring matching, accent normalization, current-context ranking, filtering, and no forced navigation during typing.

## Task 3: Artwork and spoiler interactions

Files: `src-tauri/src/journal/art.rs`, `src/journal/Artwork.tsx` and `DetailPanel.tsx`.

- [ ] Extend tracker-owned art pack to allowlisted portraits and museum/category art; keep preparation outside per-card access.
- [ ] Bound concurrent art requests, cache misses and successes, and avoid stale image flashes on profile/spoiler changes.
- [ ] Render hidden slots without identity-bearing artwork requests. Show hints only on deliberate clicks with permitted settings.
- [ ] Verify keyboard escape/focus, notes persistence, known-only related links and decoded image dimensions.

## Task 4: Verification and native delivery

- [ ] Run focused backend and UI checks, frontend production build, Rust formatting/lint and regression suite.
- [ ] Render the UI at native desktop dimensions and inspect screenshots, fixing overflow/legibility issues.
- [ ] Confirm local catalog counts and actual backup-derived progress; document any unsupported measurement explicitly.
- [ ] Replace only the tracker executable after verifying its process path, launch the native release, and report actual validated capabilities and limitations.
