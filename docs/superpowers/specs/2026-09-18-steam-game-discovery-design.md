# Steam Game Discovery Design

## Purpose

Make Mistria Tracker work when Fields of Mistria is installed outside the
default Steam folder, without asking a new player to find a technical game or
mods directory. The Tracker must locate only a valid Fields of Mistria install
and continue to treat game files and saves as read-only.

This design replaces the fixed game path currently embedded in the desktop
frontend. It is deliberately separate from the future installer, companion,
and localization work, but supplies the validated game path that those later
features need.

## Player Experience

On first launch, Tracker silently tries to locate Fields of Mistria. If it
finds one valid copy, the player sees no setup screen and the regular
readiness check starts normally.

If it cannot find a valid install, Settings shows one clear action: **Choose
Fields of Mistria folder**. The picker explains that the selected folder must
contain `assets.zip`. Once selected, Tracker validates it before using or
remembering it. A clear message explains an invalid selection and leaves
Tracker paused; it never guesses from an arbitrary directory.

The normal Settings screen does not expose Steam folders, drive letters, or
separate game/mod path fields. A later installer can use the same discovered
game location to offer the companion install destination automatically.

## Discovery Order

The native Rust backend owns all discovery and returns only a validated game
directory to the frontend.

1. **Saved validated choice.** If the player already chose a folder and it
   still contains a readable `assets.zip`, use it. If it is missing or no
   longer valid, discard it for this launch and continue.
2. **Steam configuration.** Read Steam's Windows installation location from
   the current user's Steam registry value when available. Read its
   `steamapps/libraryfolders.vdf` configuration and the default Steam library.
   For each library, accept Fields of Mistria only when both
   `steamapps/appmanifest_2142790.acf` and
   `steamapps/common/Fields of Mistria/assets.zip` exist. The app manifest
   confirms the expected Steam application; `assets.zip` confirms Tracker can
   read the installed game assets.
3. **Bounded drive fallback.** If Steam configuration is missing, unreadable,
   or incomplete, inspect only these normal Steam library shapes at the root
   of each currently mounted drive:

   - `X:\\SteamLibrary\\steamapps\\common\\Fields of Mistria`
   - `X:\\Steam\\steamapps\\common\\Fields of Mistria`
   - `X:\\Program Files (x86)\\Steam\\steamapps\\common\\Fields of Mistria`
   - `X:\\Program Files\\Steam\\steamapps\\common\\Fields of Mistria`

   Each candidate must contain `assets.zip`; candidates in a `steamapps`
   library must also have the matching `appmanifest_2142790.acf` when that
   manifest is present at the expected library level. This is a small,
   predictable set of direct path checks. Tracker never recursively scans a
   player's entire drive or personal folders.
4. **Manual fallback.** If no candidate is valid, request one folder through
   the native folder picker. Validate the chosen folder in Rust using the same
   `assets.zip` rule, then save that validated path in Tracker preferences.

When multiple valid installs are found, prefer the saved choice, then the
Steam library containing the current Steam app manifest, then the first valid
bounded fallback in stable drive-letter and candidate-order order. Settings
can later offer a single **Change game folder** action, but does not make the
player resolve a duplicate installation during first launch.

## Components and Interfaces

- `src-tauri/src/steam_discovery.rs` is a focused native module. It reads only
  Steam metadata, mounted-drive roots, and candidate `assets.zip` metadata. It
  exposes a result that distinguishes a validated directory from "not found"
  and from a non-fatal read/parse error.
- The preferences repository gains an optional validated game-directory value.
  It never stores a path until native validation succeeds. Invalid or stale
  stored paths are ignored rather than sent to the frontend.
- A Tauri command resolves the effective game directory for startup,
  readiness, catalog preparation, and live-tracking startup. Existing commands
  take a `GameAssetsPath` derived from this single resolved location rather
  than a frontend-provided constant.
- `src/lib/nativeTracker.ts` exposes typed calls for resolving and, when the
  player chooses it, validating and saving the game directory. The browser
  fallback remains inert because it has no filesystem access.
- `src/app/DesktopApp.tsx` removes the fixed `GAME` constant. It requests the
  resolved location before readiness, catalog preparation, or tracking. The
  app remains paused with an understandable Settings action when no game is
  found.
- `src/features/settings/SettingsPage.tsx` displays only the friendly missing-
  game state and one manual folder action. It never displays raw drive scans
  or accepts a text path.

The companion log stays in its existing local mod-data path. Game discovery
does not read, change, or infer a save file, and it does not install or modify
a mod.

## Safety and Failure Handling

- Every accepted game directory must be a directory with a readable
  `assets.zip`; string similarity alone is never sufficient.
- Steam registry/configuration failures and unavailable drive roots are normal
  "continue searching" conditions. They are not shown as error dialogs.
- The fallback uses direct metadata/path checks only. It does not recurse,
  index, upload, or report file names outside the fixed candidate locations.
- Folder-picker cancellation leaves the Tracker paused and does not alter any
  stored preference.
- The Tracker never writes to Steam configuration, game assets, game mods, or
  game saves. It persists only its own validated preference.
- Any changed/missing game installation invalidates the cached choice and
  reruns discovery before reading `assets.zip`.

## Test Plan

Native tests cover:

1. a valid saved game directory winning over all discovery sources;
2. default and secondary Steam libraries parsed from fixture VDF and app-
   manifest files;
3. a custom library on a non-system drive;
4. valid mounted-drive fallback shapes, stable ordering, and no recursive
   directory walk;
5. rejection of missing, unreadable, or invalid `assets.zip` candidates;
6. a missing/stale saved directory falling through to Steam discovery;
7. manual selection being rejected until valid and persisted only after valid;
8. registry/VDF permission or parsing errors degrading to the next safe step.

Frontend tests cover:

1. startup using the resolved native directory instead of the old hard-coded
   path;
2. a found installation starting the existing readiness flow;
3. a missing installation remaining paused and showing one folder action;
4. selecting a valid folder resuming readiness; and
5. cancellation or an invalid folder retaining the paused state without a
   path leak or raw system error.

Manual testing covers default Steam, a second Steam library on another drive,
an unplugged custom-drive install, and a manual selection. For every case,
confirm that the selected game folder contains `assets.zip` and that no game
save timestamp or contents change.

## Out of Scope

- Recursively searching arbitrary folders or personal data on a drive.
- Changing Steam's library configuration, game settings, game language, game
  assets, game mods, or saves.
- Installing the companion, adding a desktop shortcut, or changing the NSIS
  installer. Those consume this discovery result in later, separate work.
- Implementing the unified localization feature. That feature will consume
  this validated Steam library boundary when its existing design is approved
  for implementation.
- Auto-updates, hints, code signing, Nexus/Vortex review, and telemetry.
