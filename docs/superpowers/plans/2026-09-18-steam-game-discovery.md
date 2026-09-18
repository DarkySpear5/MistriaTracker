# Steam Game Discovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Find a valid Fields of Mistria install without a hard-coded path or an unsafe full-drive crawl.

**Architecture:** Rust owns all discovery and validation. It reads a saved validated choice, Steam library metadata, and a fixed set of Steam-shaped folders on mounted drive roots. React receives only a valid game path or no result and offers one native folder button when needed.

**Tech Stack:** Tauri 2, Rust, React, TypeScript, Vitest, Rust tests, Tauri dialog plugin.

**Spec:** `docs/superpowers/specs/2026-09-18-steam-game-discovery-design.md`

## Global Constraints

- Use only the `testing` worktree; do not touch public 0.1.3.
- Never read or write game saves, game assets, mods, or Steam configuration.
- Accept a game folder only after native code opens its direct `assets.zip` file.
- Check Steam libraries first, then only the four approved direct Steam folder shapes on mounted drive roots. Never recurse through drives.
- Keep one beginner-friendly recovery action; do not expose raw paths or scan results.
- Exclude generated `src-tauri/gen/schemas/*.json` line-ending noise from commits.

---

### Task 1: Build and test the native discovery boundary

**Files:**
- Create: `src-tauri/src/steam_discovery.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/steam_discovery.rs`

**Interfaces:**
- Produces: `DiscoveryResult::{Found(PathBuf), NotFound}` and `discover_game_directory(&DiscoverySources) -> DiscoveryResult`.
- Produces: `validate_game_directory(&Path) -> Option<PathBuf>` and an ordered, non-recursive candidate iterator.

- [ ] **Step 1: Write failing discovery tests**

```rust
#[test]
fn accepts_only_a_directory_with_readable_assets_zip() {
    let game = fixture_game_directory(true);
    assert_eq!(validate_game_directory(&game), Some(game));
    assert_eq!(validate_game_directory(Path::new("C:/missing")), None);
}

#[test]
fn finds_a_secondary_steam_library_before_drive_fallback() {
    let sources = fixture_sources_with_secondary_library();
    assert_eq!(discover_game_directory(&sources), DiscoveryResult::Found(sources.secondary_game));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test steam_discovery --lib`

Expected: FAIL because the module and its interfaces do not exist.

- [ ] **Step 3: Implement the minimum read-only discovery module**

```rust
pub const MISTRIA_APP_ID: &str = "2142790";

pub fn discover_game_directory(sources: &DiscoverySources) -> DiscoveryResult {
    sources.saved_game.as_deref().and_then(validate_game_directory)
        .or_else(|| steam_library_candidates(sources).find_map(validate_game_directory))
        .or_else(|| bounded_drive_candidates(&sources.drive_roots).find_map(validate_game_directory))
        .map_or(DiscoveryResult::NotFound, DiscoveryResult::Found)
}
```

Parse only quoted `path` fields from `steamapps/libraryfolders.vdf`. Confirm the app manifest `appmanifest_2142790.acf` for Steam-library candidates. Platform-gate real Windows registry and mounted-drive helpers, while all tests inject fixture sources.

- [ ] **Step 4: Run native discovery tests**

Run: `cargo test steam_discovery --lib`

Expected: PASS for stale saved paths, secondary libraries, malformed VDF, unreadable roots, and stable bounded candidate order.

- [ ] **Step 5: Commit this task**

Stage only `src-tauri/src/steam_discovery.rs` and `src-tauri/src/lib.rs`, then commit with message `feat: discover valid Steam game installs`.

### Task 2: Persist only a validated game folder and expose commands

**Files:**
- Modify: `src-tauri/src/app_state.rs:53-146`
- Modify: `src-tauri/src/commands.rs:1-55,471-505`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/app_state.rs`
- Test: `src-tauri/tests/commands.rs`

**Interfaces:**
- Consumes: the Task 1 validator.
- Produces: `TrackerState::resolved_game_directory() -> Result<Option<PathBuf>, TrackerStateError>`.
- Produces: `resolve_game_directory` and `save_game_directory` Tauri commands.

- [ ] **Step 1: Write failing preference/command tests**

```rust
#[test]
fn stale_saved_game_directory_falls_through_to_discovery() {
    let state = TrackerState::open(tempdir().unwrap().path()).unwrap();
    state.save_game_directory("C:/gone".into()).unwrap();
    assert_eq!(state.resolved_game_directory().unwrap(), None);
}

#[test]
fn save_game_directory_rejects_a_folder_without_assets_zip() {
    assert!(save_game_directory_value(&state(), "C:/not-a-game").is_err());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test app_state --test commands`

Expected: FAIL because the preference field, resolver, and validation command do not exist.

- [ ] **Step 3: Implement validated preferences and commands**

```rust
pub struct TrackerPreferences {
    pub language: Language,
    pub spoiler_mode: SpoilerMode,
    pub hints_enabled: bool,
    #[serde(default)]
    pub game_directory: Option<PathBuf>,
}

#[tauri::command]
pub fn resolve_game_directory(state: State<'_, TrackerState>) -> Result<Option<String>, String> {
    state.resolved_game_directory()
        .map(|path| path.map(|p| p.to_string_lossy().into_owned()))
        .map_err(|e| e.to_string())
}
```

`save_game_directory` must call Task 1 validation before writing tracker-owned preferences. Legacy preference JSON must load via `#[serde(default)]`. Never return unvalidated candidate paths to TypeScript.

- [ ] **Step 4: Run tests and commit**

Run: `cargo test --test app_state --test commands`

Expected: PASS. Stage only Task 2 files and commit with message `feat: persist validated game location`.

### Task 3: Add one folder picker and remove the hard-coded desktop path

**Files:**
- Modify: `package.json`, `pnpm-lock.yaml`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/capabilities/default.json`
- Modify: `src/lib/nativeTracker.ts`, `src/lib/nativeTracker.test.ts`
- Modify: `src/app/DesktopApp.tsx`, `src/features/settings/SettingsPage.tsx`
- Test: `src/app/App.test.tsx`, `src/features/settings/SettingsPage.test.tsx`

**Interfaces:**
- Produces: `resolveGameDirectory(): Promise<string | null>`.
- Produces: `chooseAndSaveGameDirectory(): Promise<string | null>`.
- Consumes: existing `probeReadiness`, `startLiveTracking`, and `prepare_journal` with the resolved path.

- [ ] **Step 1: Write failing bridge and UI tests**

```ts
it('uses the resolved game directory instead of the default Steam path', async () => {
  render(<DesktopApp resolveGameDirectory={async () => 'D:/Games/Fields of Mistria'} readinessProbe={probe} />);
  await waitFor(() => expect(probe).toHaveBeenCalledWith('D:/Games/Fields of Mistria', expect.any(String)));
});

it('shows one folder action when discovery returns null', () => {
  render(<SettingsPage gameLocation={{ state: 'missing' }} onChooseGameDirectory={vi.fn()} {...props} />);
  expect(screen.getByRole('button', { name: /choose fields of mistria folder/i })).toBeVisible();
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `pnpm vitest run src/lib/nativeTracker.test.ts src/app/App.test.tsx src/features/settings/SettingsPage.test.tsx`

Expected: FAIL because the bridge, Settings state, and resolved-path injection do not exist.

- [ ] **Step 3: Add the minimum-permission native dialog bridge**

```ts
export async function chooseAndSaveGameDirectory(): Promise<string | null> {
  const choice = await open({ directory: true, multiple: false, title: 'Choose Fields of Mistria folder' });
  return typeof choice === 'string'
    ? invoke<string>('save_game_directory', { gameDirectory: choice })
    : null;
}
```

Initialize the maintained Tauri dialog plugin and grant only its dialog-open permission. Remove `const GAME` from `DesktopApp`; resolve the game path before readiness, catalog preparation, and live tracking. If none is found, pause safely and show the one Settings button. Valid selection retries the same startup routine; cancellation keeps it paused.

- [ ] **Step 4: Run frontend verification and commit**

Run: `pnpm vitest run src/lib/nativeTracker.test.ts src/app/App.test.tsx src/features/settings/SettingsPage.test.tsx`

Expected: PASS.

Run: `pnpm typecheck`

Expected: PASS. Stage only Task 3 files and commit with message `feat: start tracker from discovered game install`.

### Task 4: Verify and document the feature

**Files:**
- Modify: `README.md`
- Test: existing Rust/Vitest suites

**Interfaces:**
- Consumes: all previous tasks.
- Produces: a tested, documented testing-branch feature; no public installer.

- [ ] **Step 1: Add concise beginner guidance**

```markdown
Tracker finds Fields of Mistria automatically in normal Steam libraries. If it cannot, open Settings and choose the game folder containing `assets.zip`. Tracker does not change game files or saves.
```

- [ ] **Step 2: Run full verification**

Run: `cargo test`

Expected: PASS.

Run: `pnpm test --run`

Expected: PASS.

Run: `pnpm build`

Expected: PASS.

- [ ] **Step 3: Complete manual acceptance**

1. Default Steam install: startup needs no picker.
2. Secondary-library install on another drive: startup still needs no picker.
3. Missing remembered folder: one friendly Settings action appears.
4. Picker cancellation: Tracker stays paused.
5. Valid `assets.zip` folder: readiness resumes.
6. Save timestamps before/after are identical.

- [ ] **Step 4: Commit documentation only after verification**

Stage only `README.md` and commit with message `docs: explain automatic game discovery`. Do not build, publish, submit, or replace a public installer in this task.
