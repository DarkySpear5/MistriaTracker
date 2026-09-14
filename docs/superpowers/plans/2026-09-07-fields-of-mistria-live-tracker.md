# Fields of Mistria Live Tracker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first usable Windows release of a polished, per-save Fields of Mistria tracker with passive live acquisition events, read-only save reconciliation, spoiler-safe English/French views, search, filters, notes, and compact mode.

**Architecture:** A Tauri 2 shell hosts a React/TypeScript UI while a Rust core exclusively owns game-file reads, local catalog extraction, SQLite persistence, journal ingestion, profiles, progress projection, search, and spoiler filtering. A GML MMAPI companion emits structured records through its isolated per-mod log; it uses observation-only events plus one `items.give` pass-through filter that always returns `undefined`.

**Tech Stack:** Tauri 2, Rust 1.97+ stable MSVC, React, TypeScript, Vite, pnpm, Vitest, Testing Library, SQLite via `rusqlite`, Serde, `zip`, `toml`, `flate2`, `notify`, and MMAPI/MOMI GML.

**Spec:** `docs/superpowers/specs/2026-09-07-fields-of-mistria-live-tracker-design.md`

## Global Constraints

- Target Windows first; produce an installed local desktop application, not a hosted website.
- Use Node.js 22.12.0 or newer because current Vite does not support the installed Node.js 18 runtime.
- Use the existing stable Rust MSVC toolchain, version 1.97.1 or newer.
- Use pnpm and commit `pnpm-lock.yaml` and `Cargo.lock`; do not use subscription services or paid dependencies.
- Make no outbound runtime network requests and add no telemetry.
- Never write, truncate, rename, replace, delete, repair, deserialize-and-reserialize, or restore a Fields of Mistria `.sav` file.
- Parse only tracker-owned copies created through the read-only snapshot service.
- The companion may register observation-only events and exactly one filter, `items.give`; that filter must not mutate input and must return `undefined` on every path.
- Do not use process-memory reading, DLL injection, an in-game overlay, custom game items/objects, save hooks, or MMAPI per-save storage.
- Store tracker state under `%LOCALAPPDATA%\MistriaTracker`; companion output stays under `%LOCALAPPDATA%\FieldsOfMistria\mod_data\mistria_tracker_companion`.
- Keep every game save profile isolated and never merge profiles automatically.
- Enforce spoiler rules in Rust before data reaches the frontend; aggregate totals remain visible.
- Ship English and French app resources; fall back to English per missing game field.
- Extract official text and icons locally from the user's installed game and never commit or redistribute them.
- Hints, Steam achievements, cloud sync, automatic updates, and specialized 100% categories are outside this plan.
- Real-game testing starts with a disposable character only after timestamped copies of all existing saves exist.

---

## Planned File Map

### Workspace and frontend

- `package.json`, `pnpm-lock.yaml`, `.nvmrc`: reproducible Node/pnpm workspace and scripts.
- `vite.config.ts`, `vitest.config.ts`, `tsconfig*.json`, `index.html`: frontend build and test configuration.
- `src/main.tsx`: React entry point.
- `src/app/App.tsx`, `src/app/router.tsx`: application composition and routes.
- `src/app/layout/AppShell.tsx`: sidebar, search, profile, status, and content layout.
- `src/app/styles/tokens.css`, `src/app/styles/global.css`: theme tokens and global accessibility rules.
- `src/lib/bridge.ts`: the only frontend-to-Tauri command adapter.
- `src/lib/i18n.ts`, `src/locales/en.json`, `src/locales/fr.json`: tracker-owned UI localization.
- `src/types/viewModels.ts`: frontend view-model contracts returned by Rust.
- `src/components/*`: reusable progress, card, search, filter, notification, and dialog controls.
- `src/features/{overview,collections,items,villagers,settings,notes}/*`: one feature folder per screen and its tests.

### Rust core

- `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`: desktop backend and packaging.
- `src-tauri/capabilities/main.json`: minimal Tauri permissions; no shell, HTTP, or arbitrary filesystem capability.
- `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`: service composition and Tauri startup only.
- `src-tauri/src/domain/{mod.rs,events.rs,ids.rs,progress.rs,view_models.rs}`: immutable domain contracts.
- `src-tauri/src/compatibility/{mod.rs,matrix.rs}`: verified version combinations and fail-closed decisions.
- `src-tauri/src/safety/{mod.rs,paths.rs,snapshot.rs}`: the sole game-save path and read boundary.
- `src-tauri/src/save/{mod.rs,vault.rs,evidence.rs}`: zlib vault parser and whitelisted evidence extraction.
- `src-tauri/src/catalog/{mod.rs,extract.rs,items.rs,localization.rs,availability.rs,icons.rs}`: local catalog generation.
- `src-tauri/src/persistence/{mod.rs,database.rs,migrations.rs,repository.rs}`: SQLite transactions and repositories.
- `src-tauri/migrations/0001_initial.sql`: first tracker schema.
- `src-tauri/src/profiles/{mod.rs,service.rs}`: save identity binding and active profile.
- `src-tauri/src/tracking/{mod.rs,ingest.rs,projector.rs,overrides.rs}`: accepted event/evidence projection.
- `src-tauri/src/spoilers/{mod.rs,policy.rs,search.rs}`: fact-level visibility and safe search.
- `src-tauri/src/companion/{mod.rs,journal.rs,watcher.rs}`: structured MMAPI log parsing and watching.
- `src-tauri/src/commands/{mod.rs,queries.rs,mutations.rs,settings.rs,diagnostics.rs}`: narrow Tauri commands.
- `src-tauri/src/test_support/*`: synthetic vaults, catalogs, databases, and journal records only; no real saves.

### Companion and verification

- `companion/mistria_tracker_companion/manifest.json`: MOMI metadata and required hooks.
- `companion/mistria_tracker_companion/gml/MistriaTrackerCompanion.gml`: namespaced passive companion.
- `contracts/companion-event.schema.json`: versioned journal record contract.
- `tools/companion-safety.test.ts`: conservative static checks for forbidden GML operations and filter behavior.
- `docs/manual-tests/companion-disposable-character.md`: exact real-game safety checklist.
- `docs/user-guide/{setup.md,backups.md,troubleshooting.md}`: installation and recovery guidance.

---

### Task 1: Establish the Tauri Workspace and Safety Baseline

**Files:**
- Create: `.nvmrc`
- Create: `.gitignore`
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `vitest.config.ts`
- Create: `tsconfig.json`
- Create: `tsconfig.app.json`
- Create: `index.html`
- Create: `src/main.tsx`
- Create: `src/app/App.tsx`
- Create: `src/app/App.test.tsx`
- Create: `src/test/setup.ts`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/main.json`

**Interfaces:**
- Consumes: Node.js `>=22.12.0`, pnpm, Rust stable MSVC, Windows WebView2.
- Produces: `mistria_tracker_lib::run()`, frontend `App`, scripts `pnpm test`, `pnpm typecheck`, `pnpm build`, `pnpm tauri`.

- [ ] **Step 1: Upgrade the execution shell to Node.js 22.12+ and verify prerequisites**

Run:

```powershell
node --version
pnpm --version
rustc --version
cargo --version
```

Expected: Node reports `v22.12.0` or newer, pnpm reports a version, and Rust/Cargo report `1.97.1` or newer. Stop before scaffolding if Node is still 18.

- [ ] **Step 2: Create the frontend test that proves the shell identity**

```tsx
// src/app/App.test.tsx
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from './App';

describe('App', () => {
  it('renders the tracker identity', () => {
    render(<App />);
    expect(screen.getByRole('heading', { name: 'Mistria Tracker' })).toBeVisible();
  });
});
```

- [ ] **Step 3: Install the locked free dependencies and run the failing test**

Run:

```powershell
pnpm add @tauri-apps/api react react-dom react-router-dom
pnpm add -D @tauri-apps/cli @testing-library/jest-dom @testing-library/react @types/react @types/react-dom @vitejs/plugin-react eslint jsdom prettier typescript vite vitest
pnpm test -- --run src/app/App.test.tsx
```

Expected: FAIL because `src/app/App.tsx` does not exist.

- [ ] **Step 4: Add the minimal frontend and Rust entry points**

```tsx
// src/app/App.tsx
export function App() {
  return <main><h1>Mistria Tracker</h1></main>;
}
```

```rust
// src-tauri/src/lib.rs
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("failed to run Mistria Tracker");
}
```

Configure `package.json` with `test: vitest`, `typecheck: tsc --noEmit`, `build: tsc -b && vite build`, and `tauri: tauri`. Set `.nvmrc` to `22.12.0`. Set the Tauri identifier to `app.mistriatracker.desktop`, and give `main.json` only `core:default` capability.

- [ ] **Step 5: Verify the empty product shell**

Run:

```powershell
pnpm test -- --run
pnpm typecheck
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: all commands exit 0; Vitest reports 1 passing test.

- [ ] **Step 6: Commit the workspace baseline**

```powershell
git add .nvmrc .gitignore package.json pnpm-lock.yaml vite.config.ts vitest.config.ts tsconfig.json tsconfig.app.json index.html src src-tauri
git commit -m "chore: establish tracker workspace"
```

---

### Task 2: Define Versioned Event and Compatibility Contracts

**Files:**
- Create: `contracts/companion-event.schema.json`
- Create: `src-tauri/src/domain/mod.rs`
- Create: `src-tauri/src/domain/ids.rs`
- Create: `src-tauri/src/domain/events.rs`
- Create: `src-tauri/src/compatibility/mod.rs`
- Create: `src-tauri/src/compatibility/matrix.rs`
- Create: `src-tauri/resources/compatibility.json`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: no application services.
- Produces: `ProfileId`, `ItemId`, `NpcId`, `EntityId`, `Language`, `SpoilerMode`, `EventEnvelope`, `CompanionEvent`, `GiftReaction`, `CompatibilityMatrix::decision(&VersionSet) -> CompatibilityDecision`.

- [ ] **Step 1: Write failing Serde and compatibility tests**

```rust
#[test]
fn parses_item_event_and_rejects_unknown_schema() {
    let ok = EventEnvelope::from_json(r#"{"schema_version":1,"companion_version":"0.1.0","game_version":"1.0.4","profile_id":"1849811906","session_id":"018f0000-0000-7000-8000-000000000001","sequence":7,"type":"item_obtained","payload":{"item_id":"wild_berry","count":2}}"#);
    assert!(matches!(ok.unwrap().event, CompanionEvent::ItemObtained { count: 2, .. }));
    assert!(matches!(EventEnvelope::from_json(r#"{"schema_version":99}"#), Err(EventError::UnsupportedSchema(99))));
}

#[test]
fn unknown_game_version_fails_closed() {
    let matrix = CompatibilityMatrix::fixture_for("1.0.4");
    assert_eq!(matrix.decision(&VersionSet::new("9.9.9", "0.1.0", 1)), CompatibilityDecision::PauseLiveAndImport);
}
```

- [ ] **Step 2: Run the focused tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml domain::events compatibility::matrix`

Expected: FAIL because the domain and compatibility modules are undefined.

- [ ] **Step 3: Implement the immutable event types**

```rust
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventEnvelope {
    pub schema_version: u16,
    pub companion_version: String,
    pub game_version: String,
    pub profile_id: ProfileId,
    pub session_id: Uuid,
    pub sequence: u64,
    #[serde(flatten)]
    pub event: CompanionEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum CompanionEvent {
    ProfileActivated,
    ItemObtained { item_id: ItemId, count: u32 },
    GiftGiven { npc_id: NpcId, item_id: ItemId, reaction: GiftReaction },
    MuseumDonated { item_id: ItemId, set_id: Option<String> },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language { Eng, Fra }

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum EntityId {
    Item(ItemId),
    Npc(NpcId),
    MuseumSet(String),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpoilerMode { Free, All }
```

Validate schema version before deserializing the typed payload. Newtype constructors accept only non-empty ASCII snake-case game identifiers and numeric save prefixes. `CompatibilityDecision` has `FullySupported`, `PauseLive`, and `PauseLiveAndImport` variants.

- [ ] **Step 4: Add the equivalent JSON Schema and verified matrix record**

Set `additionalProperties: false`, require every envelope field, set `schema_version` to constant `1`, require `count >= 1`, and enumerate the four event names. Add game `1.0.4`, event schema `1`, companion `0.1.x`, and save/catalog parser `1` as `probe_required: true`; Task 14 changes it to verified only after the disposable-character gate.

- [ ] **Step 5: Run contract tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml domain:: compatibility::
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: all focused tests pass and formatting is clean.

- [ ] **Step 6: Commit the contracts**

```powershell
git add contracts src-tauri/src/domain src-tauri/src/compatibility src-tauri/resources src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/lib.rs
git commit -m "feat: define tracker event contracts"
```

---

### Task 3: Build the Passive MMAPI Companion and Safety Checker

**Files:**
- Create: `companion/mistria_tracker_companion/manifest.json`
- Create: `companion/mistria_tracker_companion/gml/MistriaTrackerCompanion.gml`
- Create: `tools/companion-safety.test.ts`
- Create: `docs/manual-tests/companion-disposable-character.md`
- Modify: `package.json`

**Interfaces:**
- Consumes: event schema version `1`; MMAPI hooks `items.give`, `npc.gift_received`, `museum.donate_item`, and `game.room_changed`.
- Produces: log records prefixed `MISTRIA_TRACKER_EVENT|`; `mistria_tracker_companion_items_give(value, ctx) -> undefined`.

- [ ] **Step 1: Write the failing conservative GML safety test**

```ts
it('permits one non-mutating filter and no game/save mutation APIs', () => {
  const gml = readFileSync(COMPANION_GML, 'utf8');
  expect(gml.match(/mmapi_filter\s*\(/g)).toHaveLength(1);
  expect(gml).toContain('mmapi_filter("items.give", mistria_tracker_companion_items_give)');
  const handler = extractFunction(gml, 'mistria_tracker_companion_items_give');
  const returns = [...handler.matchAll(/\breturn\s+([^;]+);/g)].map((match) => match[1].trim());
  expect(returns.length).toBeGreaterThan(0);
  expect(new Set(returns)).toEqual(new Set(['undefined']));
  expect(handler).not.toMatch(/\b_(?:value|ctx)\.[A-Za-z_][A-Za-z0-9_]*\s*(?:=|\+=|-=|\*=|\/=|\+\+|--)/);
  for (const forbidden of ['save_game(', 'file_delete(', 'file_rename(', 'give_item(', 'modify_gold(', 'add_heart_points(', 'mmapi_modsave_register(', 'mmapi_guard(', 'mmapi_override(']) {
    expect(gml).not.toContain(forbidden);
  }
});
```

Also assert every `mmapi_on`/`mmapi_filter` hook appears in `manifest.json.requires_hooks` and every global/function uses the `mistria_tracker_companion` prefix.

- [ ] **Step 2: Run the safety test to verify it fails**

Run: `pnpm test -- --run tools/companion-safety.test.ts`

Expected: FAIL because the companion package does not exist.

- [ ] **Step 3: Create the MOMI manifest and GML boot skeleton**

```json
{
  "name": "Mistria Tracker Companion",
  "author": "Mistria Tracker",
  "version": "0.1.0",
  "description": "Passively reports local tracker events without changing saves or gameplay.",
  "minInstallerVersion": "0.14.1",
  "manifestVersion": 1,
  "requires_hooks": ["items.give", "npc.gift_received", "museum.donate_item", "game.room_changed"]
}
```

Use one latched registration function, one session-only runtime struct, and `mmapi_mod_declare("mistria_tracker_companion", "0.1.0")`. Do not call config or mod-save APIs.

- [ ] **Step 4: Implement structured logging and the pass-through filter**

```gml
function mistria_tracker_companion_items_give(_value, _ctx) {
    if (_value == undefined) return undefined;
    try {
        mistria_tracker_companion_emit("item_obtained", {
            item_id: item_id_to_string(_value.item_id),
            count: _value.count,
        });
    } catch (_error) {
        mmapi_log_error("mistria_tracker_companion", "Item event emission failed");
    }
    return undefined;
}
```

`mistria_tracker_companion_emit` reads only the basename of `Game.last_serde_path`, extracts the numeric prefix between `game-` and the next dash, increments a session sequence, serializes a plain struct, prefixes it with `MISTRIA_TRACKER_EVENT|`, writes it with `mmapi_log_info`, and calls `mmapi_log_flush`. MMAPI places those lines in `%LOCALAPPDATA%\FieldsOfMistria\mod_data\mistria_tracker_companion\logs\mistria_tracker_companion.log`. Gift handling defers one frame, then reads the matching final entry from `GAME_STATS.gifts_given` to report the reaction actually recorded by the game. Museum handling copies only the donated item and set identifiers. Every handler exits when no stable profile identifier exists.

- [ ] **Step 5: Add the exact disposable-character checklist**

The checklist requires: close the game; copy all `.sav` files to the tracker backup folder with a SHA-256 manifest; create a new disposable character; record inventory and save hashes; install through MOMI; obtain one stackable and one non-stackable item; give one liked and one disliked gift; donate one museum item; close normally; compare inventory, toast, relationship delta, museum state, and save parsing with the companion disabled; uninstall; launch again; confirm the disposable save loads. Any mismatch is a stop condition and leaves `1.0.4` marked probe-required.

- [ ] **Step 6: Run static companion verification**

Run:

```powershell
pnpm test -- --run tools/companion-safety.test.ts
pnpm typecheck
```

Expected: all companion safety tests pass. Do not perform the real-game checklist on an important character.

- [ ] **Step 7: Commit the companion package**

```powershell
git add companion contracts tools package.json pnpm-lock.yaml docs/manual-tests
git commit -m "feat: add passive tracker companion"
```

---

### Task 4: Implement Read-Only Save Snapshotting and Vault Parsing

**Files:**
- Create: `src-tauri/src/safety/mod.rs`
- Create: `src-tauri/src/safety/paths.rs`
- Create: `src-tauri/src/safety/snapshot.rs`
- Create: `src-tauri/src/save/mod.rs`
- Create: `src-tauri/src/save/vault.rs`
- Create: `src-tauri/src/save/evidence.rs`
- Create: `src-tauri/src/test_support/mod.rs`
- Create: `src-tauri/src/test_support/vault.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: `ProfileId`, verified save-parser version from `CompatibilityMatrix`.
- Produces: `SnapshotService::snapshot(&GameSavePath) -> Result<Snapshot, SnapshotError>`; `VaultReader::read<R: Read>(R) -> Result<Vault, VaultError>`; `EvidenceExtractor::extract(&Vault) -> Result<Vec<ImportedEvidence>, EvidenceError>`.

- [ ] **Step 1: Write failing snapshot and vault tests**

```rust
#[test]
fn snapshot_never_changes_source_and_parser_reads_named_json_sections() {
    let source = synthetic_save_with(&[("header", json!({"name":"Ari","farm_name":"Test"})), ("npcs", json!({}))]);
    let before = sha256(&source.path);
    let snapshot = fixture_snapshot_service().snapshot(&GameSavePath::new(&source.path).unwrap()).unwrap();
    assert_eq!(before, sha256(&source.path));
    assert_ne!(source.path, snapshot.path);
    assert_eq!(VaultReader::read(File::open(&snapshot.path).unwrap()).unwrap().section("header").unwrap()["name"], "Ari");
}

#[test]
fn changing_or_unknown_save_is_rejected_without_repair() {
    assert_matches!(snapshot_while_fixture_changes(), Err(SnapshotError::SourceChanged));
    assert_matches!(parse_fixture_version("9.9.9"), Err(EvidenceError::UnsupportedVersion(_)));
}
```

- [ ] **Step 2: Run the safety tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml safety:: save::`

Expected: FAIL because snapshot and vault types are undefined.

- [ ] **Step 3: Implement the only game-save read boundary**

```rust
pub fn copy_stable(&self, source: &GameSavePath) -> Result<Snapshot, SnapshotError> {
    let before = metadata_fingerprint(source)?;
    ensure_stable_for(self.stability_window, source, &before)?;
    let mut input = OpenOptions::new().read(true).open(source.as_path())?;
    let temp = self.backups.create_temp()?;
    std::io::copy(&mut input, &mut temp.writer)?;
    let after = metadata_fingerprint(source)?;
    if before != after { return Err(SnapshotError::SourceChanged); }
    self.backups.finalize_with_sha256(temp, before)
}
```

`GameSavePath` accepts canonical files under the configured Fields of Mistria `saves` directory with extension `.sav`; it exposes no mutation method. Only `SnapshotService` receives this type. Snapshot output is always under `%LOCALAPPDATA%\MistriaTracker\backups\game-saves`.

- [ ] **Step 4: Implement the bounded zlib vault reader**

Decode zlib, read a little-endian `u64` section count, then repeated little-endian `u64` name length, UTF-8 name, `u64` payload length, and UTF-8 JSON payload. Reject more than 256 sections, names over 256 bytes, payloads over 64 MiB, duplicate names, invalid UTF-8, invalid JSON, and trailing truncated data.

- [ ] **Step 5: Implement whitelisted evidence extraction**

Read only these verified sources: `info` for the game version; `header` for display metadata; `player.items_acquired`, `player.inventory`, and legendary-catch fields for item evidence; `npcs.*.had_arrived`, `npcs.*.gifts_given`, and `npcs.*.known_gift_preferences` for villager and exact-pair gift evidence; `gamedata.museum_progress` for donations; and `game_stats.fish_caught`, `game_stats.bugs_caught`, and `game_stats.gifts_given` for reconciliation. Establish the exact supported shapes with sanitized synthetic fixtures and never commit a real save or player-derived value. Emit `ProfileSeen`, `ItemOwned`, `NpcMet`, `GiftKnown`, `CollectionKnown`, and `MuseumDonated` evidence. Do not infer prior ownership from story progress or relationship level.

- [ ] **Step 6: Run safety and parser verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml safety:: save::
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Expected: all tests pass; Clippy reports no warnings.

- [ ] **Step 7: Commit the read-only import boundary**

```powershell
git add src-tauri/src/safety src-tauri/src/save src-tauri/src/test_support src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: add read-only save snapshots"
```

---

### Task 5: Extract the Local English/French Game Catalog

**Files:**
- Create: `src-tauri/src/catalog/mod.rs`
- Create: `src-tauri/src/catalog/extract.rs`
- Create: `src-tauri/src/catalog/items.rs`
- Create: `src-tauri/src/catalog/localization.rs`
- Create: `src-tauri/src/catalog/availability.rs`
- Create: `src-tauri/src/catalog/icons.rs`
- Create: `src-tauri/src/domain/progress.rs`
- Create: `src-tauri/src/test_support/catalog.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: read-only path to installed `assets.zip`; `ItemId` and `NpcId`.
- Produces: `CatalogExtractor::extract(&AssetsZip, &CatalogCacheDir) -> Result<Catalog, CatalogError>`; `Catalog::item(&ItemId) -> Option<&CatalogItem>`; `CatalogVersion`.

- [ ] **Step 1: Write failing catalog normalization tests**

```rust
#[test]
fn extracts_item_and_french_translation_with_english_fallback() {
    let catalog = extract_fixture_catalog();
    let item = catalog.item(&ItemId::new("paper_pondshell").unwrap()).unwrap();
    assert_eq!(item.text("fra").name, "Coquille de papier");
    assert_eq!(item.text("fra").description, item.text("eng").description);
    assert!(item.locations.contains(&LocationTag::Pond));
}

#[test]
fn numeric_ids_never_become_identity() {
    assert_eq!(extract_reordered_fixture().item_ids(), vec![ItemId::new("paper_pondshell").unwrap()]);
}
```

- [ ] **Step 2: Run the catalog tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml catalog::`

Expected: FAIL because catalog modules are undefined.

- [ ] **Step 3: Implement allowlisted ZIP and TOML extraction**

Iterate ZIP entries without extracting the archive wholesale. Parse item tables only under `assets/fiddle/items/`, French values under `assets/localization/translations/fra.meta.toml`, and supporting availability data from explicit `assets/fiddle` prefixes. Build `CatalogItem { id, text, icon_sprite, values, tags, category, seasons, weather, times, locations, museum }`. Reject path traversal and never write under the game installation.

- [ ] **Step 4: Implement localization and stable identity**

Use snake-case table keys as `ItemId`. Resolve French translation keys from the game's localization metadata; store English per field as fallback. Record `source_game_version`, source entry names, and verified aliases. Do not store numeric IDs as keys or foreign keys.

- [ ] **Step 5: Implement safe local icon caching**

Resolve each `icon_sprite` to an exact PNG ZIP entry, validate PNG signature and a 4 MiB size ceiling, then write only to a versioned temporary directory under `%LOCALAPPDATA%\MistriaTracker\cache\catalog`. Atomically rename the completed cache. Missing icons use an original tracker placeholder.

- [ ] **Step 6: Verify catalog behavior**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml catalog::
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Expected: all fixture tests pass and no game-derived output appears in `git status`.

- [ ] **Step 7: Commit catalog extraction**

```powershell
git add src-tauri/src/catalog src-tauri/src/domain/progress.rs src-tauri/src/test_support src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/Cargo.lock .gitignore
git commit -m "feat: extract local game catalog"
```

---

### Task 6: Add SQLite Profiles, Events, Evidence, Notes, and Backups

**Files:**
- Create: `src-tauri/migrations/0001_initial.sql`
- Create: `src-tauri/src/persistence/mod.rs`
- Create: `src-tauri/src/persistence/database.rs`
- Create: `src-tauri/src/persistence/migrations.rs`
- Create: `src-tauri/src/persistence/repository.rs`
- Create: `src-tauri/src/profiles/mod.rs`
- Create: `src-tauri/src/profiles/service.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: `EventEnvelope`, `ImportedEvidence`, stable IDs.
- Produces: `Database::open(AppPaths)`, `TrackerRepository::{accept_event,replace_import_evidence,save_note,append_override}`, `ProfileService::{activate,active,list}`.

- [ ] **Step 1: Write failing transaction and isolation tests**

```rust
#[test]
fn duplicate_event_is_idempotent_and_profiles_are_isolated() {
    let repo = fixture_repo();
    assert_eq!(repo.accept_event(&item_event("1849811906", "s1", 1)).unwrap(), AcceptResult::Inserted);
    assert_eq!(repo.accept_event(&item_event("1849811906", "s1", 1)).unwrap(), AcceptResult::Duplicate);
    assert_eq!(repo.events(ProfileId::new("249165455").unwrap()).unwrap().len(), 0);
}

#[test]
fn failed_migration_restores_tracker_database_only() {
    let fixture = migration_failure_fixture();
    assert!(fixture.run().is_err());
    assert_eq!(fixture.database_hash(), fixture.pre_migration_hash());
    assert!(!fixture.game_save_directory_was_touched());
}
```

- [ ] **Step 2: Run persistence tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml persistence:: profiles::`

Expected: FAIL because database services are undefined.

- [ ] **Step 3: Create the first migration**

Create tables `profiles`, `settings`, `accepted_events`, `import_batches`, `imported_evidence`, `manual_overrides`, `notes`, `journal_cursors`, and `catalog_versions`. Use `(session_id, sequence)` as the accepted-event unique key and foreign-key every profile-owned row to `profiles(id) ON DELETE RESTRICT`.

- [ ] **Step 4: Implement transaction-scoped repositories**

```rust
pub trait TrackerRepository {
    fn accept_event(&mut self, event: &EventEnvelope) -> Result<AcceptResult, RepoError>;
    fn replace_import_evidence(&mut self, profile: &ProfileId, batch: ImportBatch) -> Result<(), RepoError>;
    fn append_override(&mut self, change: ManualOverride) -> Result<OverrideId, RepoError>;
    fn save_note(&mut self, profile: &ProfileId, entity: &EntityId, text: &str) -> Result<(), RepoError>;
}
```

Reject notes over 20,000 Unicode scalar values and never place note text in logs.

- [ ] **Step 5: Implement tracker-owned backup and migration behavior**

Before migration, checkpoint SQLite, copy the database into `%LOCALAPPDATA%\MistriaTracker\backups\tracker-db`, hash it, run migrations in a transaction, and retain the ten newest tracker DB backups. Never apply this rotation to `backups\game-saves`.

- [ ] **Step 6: Verify persistence**

Run: `cargo test --manifest-path src-tauri/Cargo.toml persistence:: profiles::`

Expected: all transaction, duplicate, migration, and profile-isolation tests pass.

- [ ] **Step 7: Commit persistence**

```powershell
git add src-tauri/migrations src-tauri/src/persistence src-tauri/src/profiles src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: persist isolated tracker profiles"
```

---

### Task 7: Project Progress and Enforce Spoiler-Safe Search

**Files:**
- Create: `src-tauri/src/tracking/mod.rs`
- Create: `src-tauri/src/tracking/ingest.rs`
- Create: `src-tauri/src/tracking/projector.rs`
- Create: `src-tauri/src/tracking/overrides.rs`
- Create: `src-tauri/src/spoilers/mod.rs`
- Create: `src-tauri/src/spoilers/policy.rs`
- Create: `src-tauri/src/spoilers/search.rs`
- Create: `src-tauri/src/domain/view_models.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: repository evidence/events/overrides and `Catalog`.
- Produces: `ProgressProjector::project(&ProfileId) -> ProfileProgress`; `SpoilerPolicy::snapshot(&ProfileProgress, &Catalog, SpoilerMode, Language) -> AppSnapshot`; `SearchService::search(&AppSnapshot, &str) -> Vec<SearchResult>`.

- [ ] **Step 1: Write the spoiler matrix tests first**

```rust
#[test]
fn spoiler_free_hides_identity_but_preserves_aggregate_total() {
    let view = fixture_policy().snapshot(&progress_with_one_of_two_items(), &two_item_catalog(), SpoilerMode::Free, Language::Fra);
    assert_eq!(view.collections.items.completed, 1);
    assert_eq!(view.collections.items.total, 2);
    assert_eq!(view.items.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(), vec!["wild_berry"]);
    assert!(SearchService::search(&view, "secret").is_empty());
}

#[test]
fn gift_pair_reveals_only_after_that_pair_is_tried() {
    let view = fixture_gift_policy().for_profile_with_tried_pair("adeline", "wild_berry");
    assert_eq!(view.villager("adeline").gift("wild_berry").unwrap().reaction, GiftReaction::Liked);
    assert!(view.villager("balor").gift("wild_berry").is_none());
}
```

- [ ] **Step 2: Run focused tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml tracking:: spoilers::`

Expected: FAIL because projector and spoiler services are undefined.

- [ ] **Step 3: Implement deterministic projection**

Fold imported evidence, accepted events, and timestamp-ordered manual overrides into immutable `ProfileProgress`. Item discovery is permanent unless a later manual override hides it. Category-specific progress derives from catalog tags. Gift knowledge keys on `(profile_id, npc_id, item_id)`. Undo appends a compensating override and never deletes evidence.

- [ ] **Step 4: Implement backend-only spoiler views**

```rust
pub enum FactVisibility { Hidden, Hinted, Revealed }

pub fn item_visibility(mode: SpoilerMode, discovered: bool) -> FactVisibility {
    match (mode, discovered) {
        (SpoilerMode::All, _) | (SpoilerMode::Free, true) => FactVisibility::Revealed,
        (SpoilerMode::Free, false) => FactVisibility::Hidden,
    }
}
```

Reserve `Hinted` in the enum but never produce it in this milestone. Build totals from all catalog records, then remove hidden identities before constructing item rows, related records, filters, notifications, and search documents.

- [ ] **Step 5: Implement normalized safe search**

Search only the already-filtered `AppSnapshot`. Normalize case and diacritics for matching while returning localized original text. Include entity category and correct route. Never accept a raw catalog reference in the search API.

- [ ] **Step 6: Verify policy coverage**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml tracking:: spoilers::
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Expected: every spoiler matrix, gift-pair, aggregate, override, undo, and search test passes.

- [ ] **Step 7: Commit projection and spoiler policy**

```powershell
git add src-tauri/src/tracking src-tauri/src/spoilers src-tauri/src/domain src-tauri/src/lib.rs
git commit -m "feat: enforce spoiler-safe progress"
```

---

### Task 8: Expose Narrow Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/queries.rs`
- Create: `src-tauri/src/commands/mutations.rs`
- Create: `src-tauri/src/commands/settings.rs`
- Create: `src-tauri/src/commands/diagnostics.rs`
- Create: `src/lib/bridge.ts`
- Create: `src/types/viewModels.ts`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/capabilities/main.json`

**Interfaces:**
- Consumes: composed Rust services.
- Produces: Tauri commands `get_app_snapshot`, `search`, `save_note`, `set_manual_discovery`, `undo_last_change`, `set_spoiler_mode`, `set_language`, `set_theme`, `set_compact_mode`, and `get_redacted_diagnostics`; TypeScript `trackerBridge` with matching promises.

Define serialized `Theme { System, Light, Dark }` and `WindowMode { Full, Compact }` enums in the command settings module; neither setting changes the spoiler policy or game state.

- [ ] **Step 1: Write failing command authorization tests**

```rust
#[test]
fn commands_accept_ids_not_paths_and_diagnostics_are_redacted() {
    assert_command_has_no_path_argument(save_note_signature());
    let report = fixture_commands().get_redacted_diagnostics().unwrap();
    assert!(!report.contains("Ferme"));
    assert!(!report.contains(".sav"));
    assert!(!report.contains("my private note"));
}
```

- [ ] **Step 2: Run command tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml commands::`

Expected: FAIL because command modules do not exist.

- [ ] **Step 3: Implement command DTOs and service calls**

Every command receives stable IDs, enums, booleans, or bounded text; none receives an arbitrary filesystem path. `set_spoiler_mode(All)` requires `confirmed_irreversible_visibility: true`. Emit `tracker://changed` after a committed mutation.

- [ ] **Step 4: Implement the typed frontend bridge**

```ts
export const trackerBridge = {
  snapshot: () => invoke<AppSnapshot>('get_app_snapshot'),
  search: (query: string) => invoke<SearchResult[]>('search', { query }),
  saveNote: (entityId: string, text: string) => invoke<void>('save_note', { entityId, text }),
  setSpoilers: (mode: SpoilerMode, confirmed: boolean) => invoke<void>('set_spoiler_mode', { mode, confirmedIrreversibleVisibility: confirmed }),
};
```

- [ ] **Step 5: Lock Tauri permissions**

Keep only `core:default`, window state needed for compact mode, and app-event capability. Do not grant shell, HTTP, process, updater, unrestricted dialog, or filesystem plugins. Add a test that parses `main.json` and rejects those capability prefixes.

- [ ] **Step 6: Verify backend and bridge contracts**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml commands::
pnpm typecheck
```

Expected: Rust authorization tests and TypeScript contract checks pass.

- [ ] **Step 7: Commit the app boundary**

```powershell
git add src-tauri/src/commands src-tauri/src/lib.rs src-tauri/capabilities src/lib src/types
git commit -m "feat: expose safe tracker commands"
```

---

### Task 9: Build the Localized App Shell and Design System

**Files:**
- Create: `src/app/router.tsx`
- Create: `src/app/layout/AppShell.tsx`
- Create: `src/app/layout/AppShell.test.tsx`
- Create: `src/app/styles/tokens.css`
- Create: `src/app/styles/global.css`
- Create: `src/lib/i18n.ts`
- Create: `src/locales/en.json`
- Create: `src/locales/fr.json`
- Create: `src/components/Sidebar.tsx`
- Create: `src/components/GlobalSearch.tsx`
- Create: `src/components/ProfileBadge.tsx`
- Create: `src/components/ConnectionStatus.tsx`
- Create: `src/components/ProgressMeter.tsx`
- Create: `src/components/ConfirmDialog.tsx`
- Modify: `src/app/App.tsx`

**Interfaces:**
- Consumes: `trackerBridge.snapshot()`, `trackerBridge.search()`, localized `AppSnapshot`.
- Produces: routes `/`, `/collections`, `/villagers`, `/items`, `/settings`, `/item/:id`, `/villager/:id`; reusable accessible shell components.

- [ ] **Step 1: Write failing navigation, French, and keyboard tests**

```tsx
it('renders French navigation and keyboard-reachable search', async () => {
  renderApp({ language: 'fra' });
  expect(screen.getByRole('link', { name: 'Collections' })).toBeVisible();
  expect(screen.getByRole('link', { name: 'Villageois' })).toBeVisible();
  await userEvent.tab();
  expect(screen.getByRole('searchbox')).toHaveFocus();
});
```

- [ ] **Step 2: Run the shell tests to verify they fail**

Run: `pnpm test -- --run src/app/layout/AppShell.test.tsx`

Expected: FAIL because the shell and locale resources are undefined.

- [ ] **Step 3: Implement complete English and French shell resources**

Include navigation, search, connection states, spoiler confirmation, filters, notes, empty states, manual correction labels, themes, compact mode, backup status, diagnostics, and error copy. `i18n.ts` throws in tests when a key is absent in either language and uses English only for missing game-derived fields.

- [ ] **Step 4: Implement the semantic shell**

Use `<nav>`, `<main>`, labeled status, skip link, visible focus, and a persistent search field. `GlobalSearch` debounces 120 ms, cancels stale requests, groups results by category, and routes with each result's backend-provided route.

- [ ] **Step 5: Implement theme tokens and responsive breakpoints**

Define cream/sage/lavender/deep-blue/gold tokens, light and moonlit-dark semantic colors, focus ring, season colors, 44 px minimum interactive targets, nearest-neighbor icon rendering, and `prefers-reduced-motion`. Full mode supports 1024 px and wider; compact mode supports 420-720 px without horizontal scrolling.

- [ ] **Step 6: Verify shell quality**

Run:

```powershell
pnpm test -- --run src/app/layout/AppShell.test.tsx
pnpm typecheck
pnpm build
```

Expected: shell tests, localization completeness, typecheck, and build pass.

- [ ] **Step 7: Commit the localized design system**

```powershell
git add src/app src/components src/lib/i18n.ts src/locales
git commit -m "feat: build localized tracker shell"
```

---

### Task 10: Implement Overview, Collections, and Item Discovery Views

**Files:**
- Create: `src/features/overview/OverviewPage.tsx`
- Create: `src/features/overview/OverviewPage.test.tsx`
- Create: `src/features/collections/CollectionsPage.tsx`
- Create: `src/features/collections/CollectionFilters.tsx`
- Create: `src/features/collections/CollectionsPage.test.tsx`
- Create: `src/features/items/ItemsPage.tsx`
- Create: `src/features/items/ItemDetailPage.tsx`
- Create: `src/features/items/ItemsPage.test.tsx`
- Create: `src/components/EntityCard.tsx`
- Create: `src/components/LockedSlot.tsx`
- Create: `src/components/FilterChip.tsx`
- Create: `src/components/RecentChanges.tsx`
- Modify: `src/app/router.tsx`

**Interfaces:**
- Consumes: `AppSnapshot.overview`, `collections`, and spoiler-safe `items`; backend filter dimensions.
- Produces: navigable overview, collection grid/list, item list/detail, recent-change links, and aggregate progress displays.

- [ ] **Step 1: Write failing behavior tests**

```tsx
it('shows totals and generic locks without leaking hidden identity', () => {
  renderCollections(spoilerFreeFixture({ completed: 1, total: 2 }));
  expect(screen.getByText('1 / 2')).toBeVisible();
  expect(screen.getAllByLabelText('Undiscovered entry')).toHaveLength(1);
  expect(screen.queryByText('Secret Fish')).not.toBeInTheDocument();
});

it('combines season and location filters', async () => {
  renderItems(discoveredItemsFixture());
  await userEvent.click(screen.getByRole('button', { name: 'Spring' }));
  await userEvent.click(screen.getByRole('button', { name: 'Pond' }));
  expect(visibleItemNames()).toEqual(['Paper Pondshell']);
});
```

- [ ] **Step 2: Run feature tests to verify they fail**

Run: `pnpm test -- --run src/features/overview src/features/collections src/features/items`

Expected: FAIL because feature components do not exist.

- [ ] **Step 3: Implement Overview and recent changes**

Render active profile, connection state, recent accepted changes, museum summary, and category meters. Recent entries use routes supplied by Rust and animate only when `prefers-reduced-motion` permits.

- [ ] **Step 4: Implement collection grids and filters**

Support Fish, Insects, Artifacts, and Museum subviews; grid/list toggle; AND across filter groups and OR within one group; season, weather, time, rarity, museum set, canonical region, and environment chips. Generic locked slots have identical visuals and no tooltip.

- [ ] **Step 5: Implement Items and Item Detail**

Render only backend-provided item rows. Detail shows localized name/description, values, sources, seasons, locations, museum connection, personally discovered gift connections, evidence badge, and note entry point. Missing local icons use the original placeholder without revealing a hidden item.

- [ ] **Step 6: Verify the three screens**

Run:

```powershell
pnpm test -- --run src/features/overview src/features/collections src/features/items
pnpm typecheck
```

Expected: all focused tests pass with no hidden identity in rendered output.

- [ ] **Step 7: Commit collection and item screens**

```powershell
git add src/features/overview src/features/collections src/features/items src/components src/app/router.tsx
git commit -m "feat: add collection and item tracking views"
```

---

### Task 11: Implement Villagers, Gift Knowledge, Notes, and Corrections

**Files:**
- Create: `src/features/villagers/VillagersPage.tsx`
- Create: `src/features/villagers/VillagerDetailPage.tsx`
- Create: `src/features/villagers/GiftKnowledge.tsx`
- Create: `src/features/villagers/VillagersPage.test.tsx`
- Create: `src/features/notes/NoteEditor.tsx`
- Create: `src/features/notes/NoteEditor.test.tsx`
- Create: `src/components/DiscoveryMenu.tsx`
- Create: `src/components/UndoToast.tsx`
- Modify: `src/app/router.tsx`
- Modify: `src/lib/bridge.ts`

**Interfaces:**
- Consumes: spoiler-safe villager view models; `trackerBridge.saveNote`, `setManualDiscovery`, and `undoLastChange`.
- Produces: villager grid/detail, grouped gift knowledge, profile-specific notes, confirmed manual correction, and compensating undo.

- [ ] **Step 1: Write failing gift isolation and note tests**

```tsx
it('shows only the tested villager-item pair', () => {
  renderVillager(giftFixture({ npc: 'adeline', tried: ['wild_berry'], hidden: ['crystal_rose'] }));
  expect(screen.getByText('Wild Berry')).toBeVisible();
  expect(screen.queryByText('Crystal Rose')).not.toBeInTheDocument();
});

it('saves notes only after an explicit edit', async () => {
  const bridge = fakeBridge();
  render(<NoteEditor entityId="item:wild_berry" initialText="" bridge={bridge} />);
  await userEvent.type(screen.getByRole('textbox'), 'Keep one for Adeline');
  await userEvent.click(screen.getByRole('button', { name: 'Save note' }));
  expect(bridge.saveNote).toHaveBeenCalledWith('item:wild_berry', 'Keep one for Adeline');
});
```

- [ ] **Step 2: Run focused tests to verify they fail**

Run: `pnpm test -- --run src/features/villagers src/features/notes`

Expected: FAIL because the feature components do not exist.

- [ ] **Step 3: Implement Villagers and GiftKnowledge**

Show only revealed villagers in spoiler-free mode. Group Loved/Liked prominently and Neutral/Disliked/Hated under `Avoid or neutral`. Every row is an exact backend-provided pair; do not join against the raw item catalog in the frontend.

- [ ] **Step 4: Implement profile-specific notes**

Autosize the editor, display save/error state, enforce the backend's 20,000-character limit in UI, and preserve unsaved text if a save call fails. Never include note text in analytics or diagnostics; neither exists in this milestone.

- [ ] **Step 5: Implement manual correction and Undo**

`DiscoveryMenu` explains whether a change affects linked progress and requires confirmation. `UndoToast` calls the compensating-override command; it never deletes an event. Refresh the full safe snapshot after successful mutations.

- [ ] **Step 6: Verify villager and correction behavior**

Run:

```powershell
pnpm test -- --run src/features/villagers src/features/notes
pnpm typecheck
```

Expected: gift isolation, note failure preservation, confirmation, and undo tests pass.

- [ ] **Step 7: Commit villager and notes features**

```powershell
git add src/features/villagers src/features/notes src/components src/app/router.tsx src/lib/bridge.ts
git commit -m "feat: track villager gift knowledge and notes"
```

---

### Task 12: Watch the Companion Journal and Reconcile Active Saves

**Files:**
- Create: `src-tauri/src/companion/mod.rs`
- Create: `src-tauri/src/companion/journal.rs`
- Create: `src-tauri/src/companion/watcher.rs`
- Create: `src-tauri/src/tracking/reconcile.rs`
- Modify: `src-tauri/src/tracking/mod.rs`
- Modify: `src-tauri/src/profiles/service.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: MMAPI log lines beginning `MISTRIA_TRACKER_EVENT|`, `EventEnvelope`, repository, `SnapshotService`, `EvidenceExtractor`.
- Produces: `JournalParser::records(&str) -> JournalBatch`; `CompanionWatcher::start(...) -> WatchHandle`; `Reconciler::run(&ProfileId) -> ReconcileOutcome`.

- [ ] **Step 1: Write failing rewrite, partial-line, and profile-switch tests**

```rust
#[test]
fn rewritten_log_replays_safely_and_ignores_partial_final_line() {
    let batch = JournalParser::records(&format!("noise\n{MARKER}{event1}\n{MARKER}{partial}"));
    assert_eq!(batch.valid.len(), 1);
    assert_eq!(batch.pending_tail, format!("{MARKER}{partial}"));
    assert_eq!(ingest_twice(batch.valid[0].clone()), (AcceptResult::Inserted, AcceptResult::Duplicate));
}

#[test]
fn profile_activation_switches_without_merging() {
    let service = fixture_profile_service();
    service.ingest(profile_event("1849811906")).unwrap();
    service.ingest(item_event("1849811906", "wild_berry")).unwrap();
    service.ingest(profile_event("249165455")).unwrap();
    assert_eq!(service.active().unwrap().id.as_str(), "249165455");
    assert!(!service.active_progress().contains("wild_berry"));
}
```

- [ ] **Step 2: Run watcher tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml companion:: tracking::reconcile profiles::service`

Expected: FAIL because journal/watcher/reconciler types are undefined.

- [ ] **Step 3: Implement the MMAPI log parser**

Treat non-marker lines as ordinary mod diagnostics. Parse complete marker lines only, cap each record at 64 KiB, validate through `EventEnvelope::from_json`, and quarantine invalid complete records without logging payload contents. Session/sequence uniqueness makes full-file rewrites and restart replays idempotent.

- [ ] **Step 4: Implement the filesystem watcher**

Watch the exact companion log file resolved from the known LocalAppData root. Debounce notifications 75 ms, reopen read-only, parse the full current-session content, and persist the last observed file fingerprint. File absence means `Disconnected`; file locks and partial writes mean `Degraded`, not data deletion.

- [ ] **Step 5: Implement active-profile reconciliation**

On `profile_activated`, activate or create only that profile. After a quiet save-file period, snapshot the matching autosave and supported manual saves, parse copies, and replace the profile's current import batch transactionally. A failed import preserves the previous import batch and live events.

- [ ] **Step 6: Emit safe UI refresh events**

After an accepted event, successful profile switch, or completed reconciliation, emit `tracker://changed` containing only `{ reason, profileId }`. The frontend refetches `AppSnapshot`; never send raw catalog or journal payloads.

- [ ] **Step 7: Verify watcher and reconciliation behavior**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml companion:: tracking::reconcile profiles::service
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Expected: normal, rewrite, duplicate, truncation, missing-file, lock, partial-line, switch, and failed-reconcile tests pass.

- [ ] **Step 8: Commit live ingestion**

```powershell
git add src-tauri/src/companion src-tauri/src/tracking src-tauri/src/profiles src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: ingest live companion events"
```

---

### Task 13: Finish Settings, Compact Mode, Diagnostics, and Setup Guidance

**Files:**
- Create: `src/features/settings/SettingsPage.tsx`
- Create: `src/features/settings/SettingsPage.test.tsx`
- Create: `src/features/settings/ConnectionPanel.tsx`
- Create: `src/features/settings/BackupPanel.tsx`
- Create: `src/features/settings/DiagnosticsPanel.tsx`
- Create: `src/features/settings/ProfilePanel.tsx`
- Create: `docs/user-guide/setup.md`
- Create: `docs/user-guide/backups.md`
- Create: `docs/user-guide/troubleshooting.md`
- Modify: `src/app/App.tsx`
- Modify: `src/app/router.tsx`
- Modify: `src-tauri/src/commands/settings.rs`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Consumes: settings/diagnostic commands, profile list, compatibility decision, companion status.
- Produces: confirmed spoiler toggle, language/theme/compact preferences, backup inventory, redacted report copy, and guided MOMI setup/removal.

- [ ] **Step 1: Write failing settings and compact-mode tests**

```tsx
it('requires confirmation before enabling spoilers', async () => {
  const bridge = fakeBridge();
  renderSettings({ bridge, spoilers: 'free' });
  await userEvent.click(screen.getByRole('switch', { name: 'Show spoilers' }));
  expect(bridge.setSpoilers).not.toHaveBeenCalled();
  await userEvent.click(screen.getByRole('button', { name: 'I understand, show everything' }));
  expect(bridge.setSpoilers).toHaveBeenCalledWith('all', true);
});
```

Add a Rust window test asserting compact mode sets always-on-top, width 520, height 680, minimum width 420, and restores the prior full-window bounds when disabled.

- [ ] **Step 2: Run focused tests to verify they fail**

Run:

```powershell
pnpm test -- --run src/features/settings
cargo test --manifest-path src-tauri/Cargo.toml commands::settings
```

Expected: FAIL because settings UI and window behavior are incomplete.

- [ ] **Step 3: Implement settings and compatibility panels**

Show English/French, light/dark/system theme, spoiler mode, compact mode, profiles, game path detection, companion version/status, catalog version, and last reconciliation. Do not render a hints toggle. Unsupported versions show the exact paused subsystem and leave browsing/manual actions available.

- [ ] **Step 4: Implement backup and diagnostics panels**

List immutable game-save copy timestamps, hashes, and sizes without an automatic restore button. List rotating tracker DB backups separately. Copy diagnostics only after redaction and show the exact fields included.

- [ ] **Step 5: Implement compact window behavior**

Compact mode keeps connection, active profile, recent changes, aggregate progress, and quick search; it hides the full sidebar behind an accessible menu. Persist the preference in SQLite and use Tauri window APIs only from the Rust settings command.

- [ ] **Step 6: Write exact setup and removal guides**

`setup.md` directs the user to back up saves, install the companion archive with MOMI, launch the game with a disposable character first, and verify green connection. `backups.md` explains that tracker copies are never restored automatically. `troubleshooting.md` covers game updates, amber paused state, missing logs, MOMI removal, Steam file verification for game-install repair, and the rule never to replace a save while diagnosing.

- [ ] **Step 7: Verify settings and documentation**

Run:

```powershell
pnpm test -- --run src/features/settings
cargo test --manifest-path src-tauri/Cargo.toml commands::settings
pnpm typecheck
```

Expected: all settings, confirmation, compact-window, and diagnostics tests pass.

- [ ] **Step 8: Commit product settings and guides**

```powershell
git add src/features/settings src/app src-tauri/src/commands src-tauri/tauri.conf.json docs/user-guide
git commit -m "feat: finish tracker settings and guidance"
```

---

### Task 14: Run Full Safety, Packaging, and Disposable-Character Gates

**Files:**
- Modify: `src-tauri/resources/compatibility.json`
- Modify: `docs/manual-tests/companion-disposable-character.md`
- Create: `docs/release/first-milestone-checklist.md`
- Modify: any file implicated by a failing verification only.

**Interfaces:**
- Consumes: the completed first-milestone application, companion, tests, and approved design.
- Produces: packaged Windows installer, verified game `1.0.4` compatibility record, and a completed release checklist.

- [ ] **Step 1: Run the entire automated gate from a clean checkout**

Run:

```powershell
pnpm install --frozen-lockfile
pnpm test -- --run
pnpm typecheck
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: every command exits 0 with zero failing tests and zero warnings.

- [ ] **Step 2: Prove the save-write boundary mechanically**

Run the companion static safety suite, search production Rust/GML for forbidden save mutations, and execute snapshot integration tests against read-only fixtures. The PowerShell scan treats no match as success and prints every match for review:

```powershell
pnpm test -- --run tools/companion-safety.test.ts
$forbiddenHits = rg -n "save_game\(|file_delete\(|file_rename\(|OpenOptions::new\(\).*write\(true\)" companion src-tauri/src
if ($LASTEXITCODE -gt 1) { exit $LASTEXITCODE }
if ($forbiddenHits) { $forbiddenHits; throw "Review and remove or explicitly allowlist every production save-mutation hit." }
cargo test --manifest-path src-tauri/Cargo.toml snapshot_never_changes_source changing_or_unknown_save_is_rejected_without_repair
```

Expected: tests pass and the scan exits 0 with no production save-mutation hit. Tracker-owned backup/database writes must use different, narrowly scoped helpers and are reviewed separately.

- [ ] **Step 3: Create the immutable real-save backup set**

With Fields of Mistria closed, use the app's snapshot service to copy every `.sav` into a new timestamped `backups\game-saves` directory and generate `manifest.sha256`. Compare the source and copied file count and hashes. Do not continue unless every copy verifies.

- [ ] **Step 4: Execute the disposable-character checklist**

Perform every row in `docs/manual-tests/companion-disposable-character.md` using a newly created disposable character. Record the tested game version, MOMI version, companion version, event counts, before/after observable state, save readability, removal result, and pass/fail result. A single mismatch leaves compatibility paused and returns to the implicated task.

- [ ] **Step 5: Mark compatibility verified only after the checklist passes**

Change the `1.0.4` matrix record from `probe_required: true` to `verified: true` and include the completed checklist's commit hash. Run the compatibility tests again.

- [ ] **Step 6: Build and smoke-test the Windows package**

Run:

```powershell
pnpm tauri build
```

Expected: release build and Windows installer succeed. Install it, launch with the game closed and running, switch full/compact modes, change English/French and light/dark themes, exercise search/filters/notes/spoiler confirmation, restart, and uninstall the tracker without touching game saves.

- [ ] **Step 7: Complete the release checklist**

Record exact automated command results, package path and SHA-256, verified versions, disposable-character evidence, known safe fallbacks, companion removal steps, and any unsupported versions in `docs/release/first-milestone-checklist.md`. Do not describe an unchecked item as passing.

- [ ] **Step 8: Commit the verified first milestone**

```powershell
git add src-tauri/resources/compatibility.json docs/manual-tests docs/release
git commit -m "release: verify first tracker milestone"
```

---

## Final Definition of Done

After Task 14, compare the delivered behavior line by line with `docs/superpowers/specs/2026-09-07-fields-of-mistria-live-tracker-design.md`. The milestone is complete only when the automated suite, package smoke test, immutable backup verification, companion removal check, and disposable-character test all pass. Do not test experimental companion code on the user's important character and do not mark an unknown game version compatible.
