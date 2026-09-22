# Companion disposable-character checklist

This checklist validates the approved Fields of Mistria `1.0.5` live-tracking
surface. Use only a newly created disposable character. Do not open, copy,
select, hash, move, or otherwise inspect an important save during this test.

1. Close Fields of Mistria completely.
2. Run the Tracker installer and leave **Live tracking companion (AIM/MOMI,
   recommended)** selected. Setup should find the Steam library automatically;
   its manual fallback, if needed, selects the game root containing
   `Maybe.toml`.
3. Confirm that
   `Fields of Mistria\mods\MistriaTrackerCompanion\manifest.json` and the
   Companion GML file were copied.
4. Apply/rebuild the active profile in AIM, or run the portable MOMI tool with
   **Mistria Tracker Companion** checked and click **Install**. Close that tool
   when it finishes.
5. Launch the game normally through Steam, create or load only the disposable
   character, and change rooms once.
6. Confirm that Tracker identifies the disposable character and that its live
   status becomes active.
7. Obtain one stackable item and one non-stackable item, give one known gift,
   and donate one museum item. Confirm each event appears once in Tracker.
8. Return to the title menu. Confirm Tracker stops presenting the character as
   currently active.
9. Reload the disposable save without saving the previous session. Confirm
   Tracker reconciles to the actual save state instead of retaining unsaved
   discoveries.
10. Close the game normally, remove only the Companion test folder if cleanup
    is required, and confirm the disposable save still loads.

Any mismatch is a stop condition: do not test an important character, retain
only tracker-owned logs and test notes, and leave the affected game version
blocked until the failure is understood.
