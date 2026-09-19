# Unified Language Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the separate English/French default with one safe Auto-detect or manual language choice that controls both the Tracker interface and catalog text.

**Architecture:** Rust resolves a persisted language preference using only the `UserConfig.language` field from the already-validated Fields of Mistria Steam manifest. The frontend receives only the resolved supported locale. English and French are the only currently exposed complete Tracker dictionaries; an unsupported Steam language resolves both Tracker UI and catalog to English until reviewed full UI translations are added.

**Tech Stack:** Tauri 2, Rust, React, TypeScript, Vitest, Rust tests.

**Spec:** `docs/superpowers/specs/2026-09-17-unified-localization-design.md`

## Global Constraints

- Work only in the `testing` worktree; do not change the public 0.1.3 release.
- Never read or change a game save, game asset, mod, Steam setting, or Steam configuration other than the one manifest field needed for Auto-detect.
- A manual preference wins over Auto-detect. `auto`, missing, malformed, and unsupported values resolve to a complete supported locale or English.
- Do not present an unfinished language in Settings. The current complete dictionaries are English and French only.
- Keep frontend types and native catalog language identical. There are no separate UI and catalog language controls.
- Do not stage generated `src-tauri/gen/schemas/*.json` noise or unrelated installer edits.

---

### Task 1: Define a single locale choice and resolve the Steam manifest safely

**Files:**
- Create: `src-tauri/src/localization.rs`
- Modify: `src-tauri/src/domain/events.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/localization.rs`

**Interfaces:**
- Produces `Language::{Eng,Fra}` as the currently rendered locales.
- Produces `LanguagePreference::{Auto,Manual(Language)}` with serde values `auto`, `eng`, and `fra`.
- Produces `effective_language(preference: LanguagePreference, game_directory: Option<&Path>) -> Language`.

- [ ] **Step 1: Write failing resolver tests**

```rust
#[test]
fn auto_detects_french_only_from_the_mistria_manifest() {
    let game = fixture_game_directory_with_manifest("french");
    assert_eq!(effective_language(LanguagePreference::Auto, Some(&game)), Language::Fra);
}

#[test]
fn unsupported_or_malformed_steam_languages_fall_back_to_english() {
    assert_eq!(effective_language(LanguagePreference::Auto, Some(&fixture_game_directory_with_manifest("russian"))), Language::Eng);
    assert_eq!(effective_language(LanguagePreference::Auto, Some(&fixture_game_directory_with_manifest("<bad>"))), Language::Eng);
}
```

- [ ] **Step 2: Run the resolver tests to verify they fail**

Run: `cargo test localization --lib`

Expected: FAIL because `LanguagePreference` and the resolver do not exist.

- [ ] **Step 3: Implement the minimal read-only resolver**

```rust
pub enum LanguagePreference { Auto, Manual(Language) }

pub fn effective_language(preference: LanguagePreference, game_directory: Option<&Path>) -> Language {
    match preference {
        LanguagePreference::Manual(language) => language,
        LanguagePreference::Auto => steam_manifest_language(game_directory).unwrap_or(Language::Eng),
    }
}
```

Derive the manifest path only from the validated game's ancestor Steam library:
`<library>/steamapps/appmanifest_2142790.acf`. Read at most 1 MiB, parse only quoted `UserConfig` / `language` content, and map Steam `english` to `Eng` and `french` to `Fra`. Any unreadable, malformed, oversized, or unrecognized value returns English. Keep the parser separate from discovery and do not expose a path to the frontend.

- [ ] **Step 4: Run tests and commit**

Run: `cargo test localization --lib`

Expected: PASS for manual overrides, French detection, English detection, unsupported values, malformed manifests, and missing folders.

Stage only `src-tauri/src/localization.rs`, `src-tauri/src/domain/events.rs`, and `src-tauri/src/lib.rs`, then commit with message `feat: resolve one tracker language choice`.

### Task 2: Persist the unified choice and apply it before catalog rendering

**Files:**
- Modify: `src-tauri/src/app_state.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/tests/app_state.rs`
- Modify: `src-tauri/tests/commands.rs`

**Interfaces:**
- Consumes `LanguagePreference` and `TrackerState::resolved_game_directory()`.
- Produces `TrackerPreferences { language_preference: LanguagePreference, ... }` and `TrackerState::effective_language() -> Result<Language, TrackerStateError>`.
- `get_preferences` exposes only the preference (`auto`, `eng`, or `fra`) and effective locale (`eng` or `fra`), never a game path.

- [ ] **Step 1: Write failing state and command tests**

```rust
#[test]
fn legacy_preferences_become_auto_detect_without_losing_safety_defaults() {
    let state = state_with_legacy_preferences("{\"language\":\"fra\",\"spoiler_mode\":\"free\",\"hints_enabled\":false}");
    assert_eq!(state.preferences().unwrap().language_preference, LanguagePreference::Auto);
}

#[test]
fn preferences_value_reports_one_effective_language_without_a_game_path() {
    assert_eq!(preferences_value(&state).unwrap()["effective_language"], "eng");
    assert!(preferences_value(&state).unwrap().get("game_directory").is_none());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test app_state --test commands`

Expected: FAIL because the language preference and effective-locale boundary do not exist.

- [ ] **Step 3: Implement safe preference migration and resolution**

Move the persisted field from `language: Language` to
`language_preference: LanguagePreference`, with `#[serde(default)]` returning
`Auto`. Keep a custom legacy deserialize path that turns prior `language`
values into `Auto`, so existing users receive the recommended behavior rather
than a surprise forced override. Resolve the effective language from the
validated saved/discovered game directory before returning preferences and
before building a catalog snapshot. The existing English/French catalog fields
continue to select from the same effective `Language`; unknown game languages
remain English for both layers.

- [ ] **Step 4: Run tests and commit**

Run: `cargo test --test app_state --test commands`

Expected: PASS for legacy migration, manual overrides, Auto-detect fallback, and path-free command values.

Stage only Task 2 files and commit with message `feat: persist unified language preference`.

### Task 3: Replace paired UI copy with complete locale dictionaries and one Settings control

**Files:**
- Create: `src/i18n/registry.ts`
- Create: `src/i18n/en.ts`
- Create: `src/i18n/fr.ts`
- Create: `src/i18n/index.ts`
- Modify: `src/journal/copy.ts`
- Modify: `src/journal/types.ts`
- Modify: `src/app/DesktopApp.tsx`
- Modify: `src/lib/nativeTracker.ts`
- Modify: `src/app/App.test.tsx`
- Test: `src/i18n/index.test.ts`
- Test: `src/app/App.test.tsx`

**Interfaces:**
- Produces `SupportedLocale = 'eng' | 'fra'`, `LanguagePreference = 'auto' | SupportedLocale`, and `translate(locale, key, variables?)`.
- Consumes `NativePreferences.effective_language` and `language_preference`.
- Produces one Settings language selector: Auto-detect, English, Français.

- [ ] **Step 1: Write failing dictionary and UI tests**

```ts
it('falls back to English instead of displaying an untranslated key', () => {
  expect(translate('fra', 'missingKey')).toBe(translate('eng', 'missingKey'));
});

it('uses the detected locale for both catalog text and Tracker controls', async () => {
  render(<App nativeRuntime={() => true} />);
  await waitFor(() => expect(screen.getByRole('button', { name: 'Réglages' })).toBeVisible());
});

it('persists one manual language choice and never shows a second catalog selector', async () => {
  // Native preference returns auto; choosing Français saves `language_preference: 'fra'`.
});
```

- [ ] **Step 2: Run frontend tests to verify they fail**

Run: `node node_modules/vitest/vitest.mjs --run src/i18n/index.test.ts src/app/App.test.tsx`

Expected: FAIL because dictionary modules and the single preference control do not exist.

- [ ] **Step 3: Implement dictionaries and one selector**

Move each current English/French entry in `src/journal/copy.ts` into complete
`en.ts` / `fr.ts` dictionaries with a typed English key set. Implement
`translate` so a missing French entry falls back to English and never returns
the raw key. Remove component-local language conditionals while preserving
existing English/French wording. In Settings, replace the two plain language
buttons with three mutually exclusive choices: **Auto-detect (recommended)**,
**English**, and **Français**. The initial selected state comes from the native
preference; changing it sends the one `language_preference` field and refreshes
the same catalog snapshot. Do not show Russian, Spanish, Chinese, Japanese, or
Korean until each has a complete reviewed Tracker dictionary.

- [ ] **Step 4: Run frontend checks and commit**

Run: `node node_modules/vitest/vitest.mjs --run src/i18n/index.test.ts src/app/App.test.tsx src/lib/nativeTracker.test.ts`

Expected: PASS.

Run: `node node_modules/typescript/bin/tsc --noEmit -p tsconfig.app.json`

Expected: PASS.

Stage only Task 3 files and commit with message `feat: unify tracker language selection`.

### Task 4: Document the current complete-locale policy and verify the branch

**Files:**
- Modify: `README.md`
- Modify: `docs/ROADMAP.md`

- [ ] **Step 1: Add concise user-facing language guidance**

Add: “Language uses Auto-detect by default. English and French are currently
fully translated; unsupported game languages use English consistently until a
complete reviewed Tracker translation is available.”

- [ ] **Step 2: Run full verification**

Run: `cargo test`

Expected: PASS.

Run: `node node_modules/vitest/vitest.mjs --run`

Expected: PASS.

Run: `node node_modules/typescript/bin/tsc -b` followed by `node node_modules/vite/bin/vite.js build`

Expected: PASS.

- [ ] **Step 3: Commit documentation**

Stage only `README.md` and `docs/ROADMAP.md`, then commit with message
`docs: explain unified language support`.

## Plan Self-Review

- Spec coverage: Tasks 1-3 implement one effective language, safe Steam-manifest Auto-detect, native-only filesystem reads, English fallback, and UI translation isolation. Task 4 documents the intentionally limited complete-locale release.
- Scope: reviewed interface translations for the remaining six official game languages are explicitly deferred; exposing partial or machine-translated UI would violate the design and user experience.
- Type consistency: Rust persists `LanguagePreference` and resolves `Language`; TypeScript persists `LanguagePreference` and renders `SupportedLocale`. Both use `auto`, `eng`, and `fra` wire values.
