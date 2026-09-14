# Fields of Mistria Live Tracker Design

**Date:** 2026-09-07
**Status:** Approved in conversation; awaiting review of this written specification

## Summary

Build a polished, local-first Windows desktop tracker for Fields of Mistria. A passive MMAPI companion mod observes supported game events and writes an isolated event journal. The companion uses observation-only event hooks plus the single `items.give` filter required for universal item acquisition; that handler always returns `undefined` so the original grant remains unchanged. A Tauri application ingests those events, reconciles against read-only save snapshots, and presents per-save progress without exposing undiscovered content when spoiler-free mode is active.

The first milestone covers villagers and personally discovered gift preferences, fish, insects, artifacts, museum progress, discovery of any locally cataloged item obtained during play, global search, season and location filters, descriptions, notes, English and French, spoiler controls, and a compact always-on-top window. Categories outside the focused collection screens receive item discovery and detail pages but not specialized completion workflows in this milestone. Later milestones extend the same model toward full 100% completion, optional hints, and Steam achievements.

## Product Principles

1. **Save safety is non-negotiable.** Tracking may pause or fail, but the app and companion must never repair, modify, replace, rename, or delete a Fields of Mistria save.
2. **Spoiler protection is a data-access rule.** Hidden facts must not leak through search, autocomplete, filters, totals beyond approved aggregate counts, recent history, notifications, or linked records.
3. **Local-first means fully local.** The first milestone needs no account, domain, hosted service, telemetry, or network connection.
4. **Progress belongs to a save.** Each character/save has an isolated tracker profile and event history. Profiles are never merged automatically.
5. **Game updates fail closed.** An unknown game, save, catalog, or companion version pauses the affected synchronization path instead of guessing.
6. **The app remains extensible.** Hints, additional categories, full completion, more languages, and Steam achievements can be added without changing the core identity or spoiler model.

## First-Milestone Scope

### Included

- Tauri 2 desktop application for Windows.
- React and TypeScript user interface with a Rust safety and persistence core.
- MMAPI-based GML companion using observation-only events and one pass-through `items.give` filter that always returns `undefined`.
- Automatic active-save detection and a separate tracker profile for every save.
- Read-only import and reconciliation from supported save versions.
- Timestamped pre-import copies of game saves stored outside the game data directory.
- Live discovery for supported item pickups, fish catches, insect catches, artifact finds, gifts, and museum donations.
- Overview, Collections, Villagers, Items, and Settings screens.
- Global spoiler-aware search.
- Item descriptions, sources, values, availability, notes, and known relationships.
- Season, weather, time, category, rarity, museum set, and normalized location filters.
- English and French app UI and locally extracted game content.
- Light, dark, full-window, and compact always-on-top presentations.
- Manual mark/unmark and undo for evidence the game no longer retains.
- Diagnostics that contain no save contents.

### Explicitly deferred

- The hints feature and authored hint content.
- Specialized completion workflows for quests, achievements, recipes, crops, forageables, food, crafting materials, equipment, cosmetics, animals, and other 100% categories. Obtained objects from these categories may still appear as ordinary discovered items.
- Steam achievement synchronization.
- Languages other than English and French.
- Transparent or interactive overlays on top of the game.
- Accounts, cloud sync, mobile clients, and web hosting.
- Automatic app updates and automatic companion installation.

## Technology Choice

The desktop application uses Tauri 2. The UI uses React and TypeScript for a refined, responsive interface; the trusted backend uses Rust for save snapshotting, parsing, event validation, SQLite access, catalog extraction, and spoiler-safe queries. The result is an installed local desktop application, not a hosted website.

The companion uses the current MMAPI hook framework and contains GML only. Installation is guided through the established Mods of Mistria Installer (MOMI). The first milestone does not implement its own patcher and does not rewrite `assets.zip` directly. MMAPI currently exposes universal item grants as the `items.give` filter rather than an event. The companion registers that one filter solely to copy the incoming item identifier and count, then explicitly returns `undefined`, which MMAPI defines as preserving the current value.

SQLite stores tracker-owned state under `%LOCALAPPDATA%\MistriaTracker`. The app uses a local WebView for presentation but grants the frontend no direct access to arbitrary files. Narrow Tauri commands expose only validated tracker operations.

## System Architecture

### Desktop UI

The frontend renders navigation, lists, detail views, filters, notes, notifications, settings, and diagnostics. It never reads game saves, the event journal, or SQLite directly. It requests spoiler-safe view models from the Rust core.

### Rust core

The Rust core owns these bounded services:

- **Profile service:** binds a stable game-save identifier to one tracker profile and selects the active profile from companion session events.
- **Snapshot service:** copies stable save bytes using read-only source access and passes only the copy to the parser.
- **Save parser:** reads explicitly supported save versions and emits whitelisted evidence records.
- **Event ingester:** validates, deduplicates, orders, and persists companion events.
- **Progress projector:** derives current discovery and completion state from imported evidence, live events, and explicit manual overrides.
- **Spoiler policy:** converts catalog and progress data into view models that contain only facts allowed by current settings.
- **Catalog service:** extracts and normalizes supported facts and localizations from the installed game into a versioned local cache.
- **Search service:** indexes only the view permitted by the spoiler policy.
- **Notes service:** stores user notes against stable entity identifiers without exposing hidden entities.
- **Backup and migration service:** manages tracker-database migrations, tracker backups, and immutable pre-import game-save copies.

Each service has a narrow interface and can be tested without launching the UI or game.

### Companion mod

The companion registers observation-only event callbacks and exactly one filter callback: `items.give`. The `items.give` handler treats the incoming value as immutable, copies only `item_id` and `count` into a plain event record, and returns `undefined` on every branch, including invalid input and journal failure. It does not register any other filters, guards, overrides, custom items, custom objects, save serialization hooks, or persistent per-save MMAPI data. It does not call functions that alter inventory, relationships, collections, museum state, time, or progression.

After MMAPI initialization and compatibility checks, the companion observes supported events and appends prefixed, newline-delimited records to MMAPI's guaranteed per-mod log:

`%LOCALAPPDATA%\FieldsOfMistria\mod_data\mistria_tracker_companion\logs\mistria_tracker_companion.log`

Every tracker record begins with `MISTRIA_TRACKER_EVENT|`. The app treats the log as a rewritable journal: it rescans safely after rotation or rewrite and deduplicates records by session identifier plus sequence number.

The companion never constructs or opens a path containing the game's `saves` directory. Its runtime state is session-only. Uninstalling it leaves no custom objects, items, or tracker state serialized into a save.

### Data flow

1. Loading a save causes the companion to emit a profile-activation event.
2. The Rust core binds that stable save identifier to the corresponding tracker profile.
3. Gameplay events are appended to the companion's isolated journal.
4. The Rust core watches the journal, validates records, and stores accepted events in SQLite.
5. The progress projector updates derived state in one transaction.
6. The spoiler policy produces a safe UI update.
7. The UI animates the changed card and offers a link to its detail page.
8. Periodic read-only reconciliation fills supported gaps without changing the game save.

## Save-Safety Boundary

### Prohibited operations

Neither component may write, truncate, rename, replace, delete, repair, deserialize-and-reserialize, or restore a `.sav` file. There is no automatic restoration path. Unknown or malformed saves produce a diagnostic and are left untouched. The one permitted filter callback must return `undefined` and must never mutate its incoming value or context.

The companion has no save-directory code. The Rust core centralizes all game-save access in the snapshot service, whose public interface exposes only a read-and-copy operation. No other service receives an original game-save path.

### Stable snapshot procedure

1. Record the source file's identity, length, and last-write time.
2. Wait until those values are stable across two observations.
3. Open the source with read-only access and file sharing compatible with the game.
4. Copy bytes to a uniquely named temporary file inside the tracker's backup area.
5. Record the source metadata again.
6. If the source changed, discard only the tracker-owned temporary copy and retry later.
7. Hash the completed copy, store its manifest, atomically finalize it inside the tracker backup area, and parse only that copy.

The first supported import for a profile creates a timestamped copy of every `.sav` file in the game's save directory under `%LOCALAPPDATA%\MistriaTracker\backups\game-saves\<timestamp>\`. These game-save copies are never deleted automatically and are never restored automatically.

### Tracker persistence

Tracker state is stored in `%LOCALAPPDATA%\MistriaTracker\data\tracker.sqlite3`. Mutations use SQLite transactions. The application makes rotating tracker-database backups before schema migrations and after clean checkpoints. Rotation applies only to tracker-owned database backups, never to copied game saves.

An append-only accepted-event table is the audit source for live progress. Derived tables can be rebuilt from accepted events, the latest supported import evidence, and manual overrides.

## Companion Event Contract

Every record contains:

- Event schema version.
- Companion version.
- Game version.
- Stable save/profile identifier.
- Random session identifier.
- Monotonically increasing sequence number within the session.
- Event type.
- Minimal typed payload.

The unique event identity is the session identifier plus sequence number. Replaying, duplicating, or rereading a journal therefore cannot apply an event twice.

First-milestone event types are:

- `profile_activated`
- `item_obtained`
- `gift_given`
- `museum_donated`

Fish, insect, artifact, forageable, crop, and other category progress derives from `item_obtained` plus the locally extracted catalog classification, avoiding extra gameplay hooks and duplicate acquisition events. `gift_given` includes villager identifier, item identifier, and the reaction class actually produced by the game. It never queries the full gift table. `museum_donated` includes item and set identifiers from the completed donation event.

Records with an unknown schema, version, event type, identifier, or invalid payload are quarantined in tracker-owned diagnostics and never projected into progress.

The journal is transport, not authoritative game data. If it is missing, truncated, delayed, or locked, the app shows degraded status and later reconciles from supported save evidence. Tracker ingestion never blocks a game callback. The `items.give` handler returns `undefined` even when journal emission fails, so tracking failure cannot replace, cancel, or alter the item grant.

## Compatibility and Failure Behavior

The companion and app exchange their versions through journal session records. A compatibility matrix in the app identifies verified combinations of game, companion, event schema, save parser, and catalog schema versions.

For an unverified game version:

- The companion's handlers return without emitting gameplay events.
- The app shows an amber `Update detected - live tracking paused` state.
- Existing tracker data remains available.
- Manual tracking remains available.
- Save import is disabled unless that save version is independently verified.

A broken event callback is allowed to fail inside MMAPI error isolation and cannot veto or replace game behavior. The `items.give` filter catches journal errors locally and returns `undefined`; MMAPI also preserves the current value when a filter throws. No unsafe memory-reading or process-injection fallback is permitted. If a passive hook is unavailable, that event uses supported read-only reconciliation or manual marking until a safe hook is verified.

## Profile Model

Every Fields of Mistria save identifier maps to exactly one tracker profile. The profile stores display metadata such as character and farm name separately from its stable identifier. Display-name changes do not create new profiles.

When the game loads another save, `profile_activated` switches the UI automatically. Concurrent or ambiguous identifiers pause automatic switching and ask the user to select from already identified profiles. Profiles are never merged automatically. Manual merging is outside the first milestone.

Notes are profile-specific in the first milestone. Global/shared notes are deferred.

## Catalog and Localization

The catalog service reads the locally installed `assets.zip` without modifying it. It extracts only the whitelisted fields needed by the tracker, including stable textual identifiers, names, descriptions, tags, values, availability, museum relationships, villager facts, and references to locally installed icons.

English source content and French translations are stored in a versioned tracker cache. Tracker-owned interface text has complete English and French resource files. The initial app language follows the game's `language` setting when it is `eng` or `fra`; the user can switch the tracker independently. A missing translated field falls back to English and is marked in diagnostics, not in the normal UI.

Stable snake-case text identifiers are primary keys. Numeric game identifiers may be cached only as version-stamped diagnostic data and are never used as database keys or foreign keys. Catalog updates preserve aliases for verified renames. If an entity disappears from a later game version, its progress and notes are archived rather than deleted.

Normalized filter dimensions are:

- Category and subtype.
- Season.
- Weather class.
- Time window.
- Rarity.
- Museum wing and set.
- Canonical region, plus a specific room only when the extracted catalog explicitly identifies that room; otherwise the region is the most precise stored location.
- Acquisition environment, including ocean, river, pond, beach, surface, mines, and other verified sources.

Game text and images remain in a local cache generated from the user's installed copy. The repository and any future public installer contain extraction logic and original tracker assets, not redistributed proprietary game content. A public release receives a separate legal and attribution review before distribution.

## Spoiler Policy

Spoiler state applies per fact, not per page.

### Spoiler-free mode

- An item identity becomes visible when acquisition is supported by a live event, imported save evidence, or a manual override.
- A revealed item's description, value, seasons, locations, availability, and acquisition methods become visible.
- A villager identity becomes visible after supported evidence shows that profile has met the villager.
- A villager/item gift rating becomes visible only after that exact combination was tried or is explicitly recorded as known in the save.
- Loved and Liked gifts appear prominently. Neutral, Disliked, and Hated gifts appear in a compact `Avoid or neutral` area.
- Museum donation and discovery status reflect only actual profile evidence.
- Unknown identities, names, icons, descriptions, relationships, and acquisition instructions are excluded from UI responses and the search index.
- Aggregate counts are permitted. Overall, category, filtered, and museum-set views may show values such as `24 / 120`, `Fish 18 / 55`, or `Ocean 7 / 19`.
- Unknown collection entries may be represented by identical generic locked slots. They reveal no identifying silhouette, name, icon, or item-specific metadata.

### Spoiler-enabled mode

The full locally extracted catalog is visible. Independently tracked badges distinguish discovered, obtained, donated, known, and manually marked facts from catalog knowledge.

Enabling spoilers requires a confirmation explaining that the setting can be turned off again but already seen information cannot be forgotten.

### Future hints mode

The data model reserves `hidden`, `hinted`, and `revealed` visibility states. The first milestone implements only `hidden` and `revealed` and exposes no hints setting. A later hints feature may show a heavily obscured generic entry and an original, vague English/French acquisition hint without revealing the exact identity or directions.

### Leak prevention

The Rust spoiler-policy service filters records before they reach the frontend. Search, autocomplete, related-item queries, filter options, event notifications, recent history, notes navigation, and empty-state text all consume filtered view models. The frontend is not trusted to hide forbidden fields with CSS.

## Progress Semantics and Manual Corrections

Discovery is permanent for a tracker profile unless the user explicitly undoes or unmarks it. Selling, gifting, consuming, donating, or losing an item does not make it undiscovered.

Imported evidence reveals only facts explicitly supported by a parsed save section. Story position, relationship level, inventory adjacency, and likely encounters are never treated as proof.

Manual `Mark discovered` and `Mark not discovered` actions require a clear confirmation when they affect linked progress. They create timestamped override records rather than deleting automatic evidence. The UI normally shows the final state; diagnostics can distinguish live, imported, and manual sources.

Recent live changes offer Undo. Undo creates a compensating manual override and never edits or removes the original audit event.

## User Experience

### Navigation

- **Overview:** active profile, connection and compatibility status, recent discoveries, category progress, and museum progress.
- **Collections:** fish, insects, artifacts, and museum sets with grid/list display and availability filters.
- **Villagers:** revealed villager grid, relationship information supported by the save, birthdays, discovered gift ratings, and notes.
- **Items:** all revealed items in spoiler-free mode or the complete catalog in spoiler-enabled mode, with descriptions, sources, availability, value, connections, and notes.
- **Settings:** language, spoiler mode, compact mode, theme, profile management, backups, connection setup, and diagnostics.

A global search field remains available throughout the app. Search results span supported items, villagers, and collection records and route to the correct detail view.

### Live feedback

When a valid event changes visible progress:

1. The connection indicator pulses briefly.
2. The affected card unlocks or updates with reduced, non-blocking motion.
3. A small in-app notification explains the observed change.
4. Selecting the notification opens the affected detail view.

Duplicate or spoiler-ineligible events do not create misleading notifications.

### Full and compact windows

The full window targets information-rich desktop use. Compact mode is a normal resizable always-on-top window, not a game overlay. It focuses on connection status, recent discoveries, concise progress, and quick search. Both modes preserve the active profile and spoiler policy.

## Visual Design

The product uses an original modern magical-journal aesthetic rather than copying the game's menus. The palette combines warm cream, sage, lavender, deep blue, and restrained gold. A comfortable light theme and moonlit dark theme share the same hierarchy and contrast standards.

Locally extracted pixel-art icons render with nearest-neighbor scaling. Modern cards, chips, inputs, and data-dense lists provide the surrounding structure. Body typography prioritizes readability and French accent coverage over decorative styling.

Spring, summer, fall, and winter use consistent accent colors across filters and availability markers. Status is never communicated by color alone. Keyboard navigation, visible focus, scalable text, reduced-motion support, and useful contrast are required. Collection screens favor scanning density; detail screens allow richer art, descriptions, connections, and notes.

## Privacy and Networking

The first milestone makes no outbound network requests and contains no telemetry. Companion communication is the local event journal, not a socket exposed to the network. Diagnostics remain local and exclude save contents, character names by default, and note text. A user may explicitly copy a redacted diagnostic report.

Future Steam achievement support is a separate opt-in integration with its own permission, privacy, and failure model. It cannot become a dependency of local tracking.

## Testing Strategy

### Unit tests

- Save parser behavior for supported, truncated, corrupted, changing, and unknown-version copies.
- Snapshot service source access is read-only and cannot expose a write operation.
- Event schema validation, deduplication, ordering, quarantine, and replay.
- Static and runtime companion checks proving the `items.give` handler never mutates its input and returns `undefined` for valid, invalid, and journal-failure paths.
- Projection from live, imported, and manual evidence.
- Profile isolation and automatic switching.
- Spoiler-policy matrices for each entity and fact type.
- Search and autocomplete exclusion of hidden facts.
- Aggregate completion totals without identity leakage.
- English/French selection, fallback, accents, and sorting.
- Catalog aliases, missing entities, and game-version migrations.
- Tracker database transactions, migrations, backup rotation, and rebuilds.

### Integration tests

- Simulated journal streams covering normal play, restart, partial final lines, truncation, file locks, session rollover, and duplicated records.
- Read-only snapshot creation while a fixture writer changes the source.
- End-to-end event-to-view updates for every first-milestone event type.
- Tauri command authorization showing the frontend cannot request arbitrary paths.
- Unsupported game and companion version handshakes.
- Disposable-character verification that item identifiers, counts, toast behavior, inventory results, and save results are identical with the pass-through filter enabled and disabled.
- Removal of the companion with no tracker-specific serialized game state.

### UI and accessibility tests

- Full and compact layouts at supported Windows scaling levels.
- Light and dark themes.
- Keyboard-only operation and visible focus.
- Reduced-motion behavior.
- Long French strings and mixed fallback content.
- Spoiler confirmation and manual-correction confirmations.
- Search, filters, notes, event notifications, and profile switching.

### Real-game verification

Before the first real-game test, make timestamped copies of all existing saves. Test experimental companion builds first with a separate disposable game character, not the user's important character. Verify each passive hook individually and confirm the game behaves identically with the tracker closed, connected, disconnected, and after companion removal.

## Acceptance Criteria

The first milestone is complete when:

- The app and guided companion installation work on the supported Windows and game versions.
- The app identifies the active save profile and never merges profiles automatically.
- Existing supported evidence imports from a read-only, stable snapshot.
- Supported live item, fish, insect, artifact, gift, and museum events appear in the tracker within one second under normal local conditions.
- Restarting the app or game loses no accepted tracker progress.
- Duplicate and malformed events do not change progress.
- Search, filters, descriptions, notes, aggregate totals, English/French, themes, spoiler mode, and compact mode meet the rules in this specification.
- Automated tests prove there is no game-save write interface.
- Unsupported versions pause safely and preserve existing tracker data.
- Removing the companion leaves saves free of custom tracker objects and state.
- A clean build, automated test suite, packaged-app smoke test, and disposable-character real-game checklist all pass.

## Delivery Sequence

Implementation planning will decompose this design into these ordered outcomes:

1. Verify passive MMAPI hook coverage with a disposable-character probe and define the final event mapping.
2. Establish the safety boundaries, fixtures, snapshot service, event contract, and version gates.
3. Build catalog extraction and English/French normalization.
4. Build profile, SQLite, evidence, projection, and spoiler-policy services test-first.
5. Build the desktop navigation, search, filters, detail views, notes, and settings.
6. Connect and verify the companion journal and live UI updates.
7. Complete compact mode, themes, accessibility, packaging, diagnostics, and removal guidance.
8. Run the full acceptance and disposable-character verification gates.

Later specifications will cover hints, additional completion categories, Steam achievements, more languages, and public distribution.

## Primary Risks and Mitigations

- **A required event lacks a stable passive hook.** Use read-only reconciliation or manual marking until a verified MMAPI seam exists; do not substitute mutation hooks or memory injection.
- **A game update changes hooks, assets, or saves.** Version gates disable only the affected path, preserve tracker data, and require explicit compatibility verification.
- **An existing save lacks historical acquisition evidence.** Import only explicit evidence and provide auditable manual marking.
- **French fields are incomplete.** Fall back per field to English while retaining the stable identity and reporting missing coverage in diagnostics.
- **A public release creates asset-distribution concerns.** Distribute original code and extraction rules only; generate game-derived caches locally and conduct a separate legal review.
- **The event journal is interrupted.** Deduplicate and replay valid records, quarantine malformed records, and reconcile from supported read-only evidence.

## Locked Decisions

- Tauri desktop application, not a website.
- Passive companion mod, not process-memory reading; the only filter is `items.give`, and it always returns `undefined` without mutating its input.
- No in-game overlay.
- Per-save profiles with automatic switching.
- Spoiler-free item reveal on acquisition and gift reveal per tested villager/item pair.
- Aggregate completion totals remain visible in spoiler-free mode.
- Optional hints are designed for but not implemented in the first milestone.
- English and French ship first.
- The first milestone is focused; full 100% completion and Steam achievements follow later.
