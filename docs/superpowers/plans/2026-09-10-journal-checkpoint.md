# Desktop journal checkpoint — 80% estimate

This percentage refers to the agreed redesign, not all future game/Steam completion tracking.

## Verified in this checkpoint

- Fixed desktop shell; Overview, Museum, Villagers, Encyclopaedia and Settings.
- Actual backup-derived progress and game-defined museum sets, portraits and gift tables.
- Separate acquired items, donated items, learned recipes and discovered gift preferences.
- Partial, accent-insensitive multi-result search with context-aware selection.
- Museum donation filters, villager encounter filters, category availability filters and sorting.
- Notes confirm before in-app navigation discards an unsaved edit.
- Opaque artwork tokens, backend-generated black PNG silhouettes, bounded lazy loading and separate hidden/revealed caches.
- One-frame extraction using animation metadata; portrait transparent padding trimmed.
- Correct French game translation keys.
- Automatic startup import; cheap log-availability recheck when companion starts later.
- 76 frontend tests passed; Rust regression suite passed before the final reconnect command; focused journal tests and Rust clippy passed after portrait extraction.
- Real approved Amelia backup read-only test passed and checked source immutability.
- Native release built successfully; deployment/launch verification still pending at writing.

## Still open — do not call this 100%

- Animals and Cosmetics catalog coverage, with verified unlock evidence and appropriate artwork.
- Audit recipe-key aliases/default recipes before claiming a full learned-recipe total.
- Audit art availability and metadata coverage beyond the currently rendered screens.
- Known imported gift preferences should open details even where no item acquisition was recorded.
- Native close/profile-switch behavior while a note is dirty or still loading needs additional coverage.
- Final minimum-size visual pass and native runtime smoke test.
- Independent Luna Medium review requested for startup, notes and artwork caching.

No game save, game archive or companion files were edited by this redesign checkpoint. Generated caches and diagnostic previews are tracker-owned. Steam achievements remain a future feature and are explicitly separate from collection percentages.
