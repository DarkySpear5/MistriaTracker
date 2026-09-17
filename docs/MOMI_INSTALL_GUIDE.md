# Mistria Tracker — MOMI installation guide

## What is required

- Windows 10 or Windows 11 (64-bit)
- Fields of Mistria installed and updated
- MOMI/Mods of Mistria 0.14.1 or newer
- The Mistria Tracker desktop app installer, or its portable ZIP
- The companion mod copied into the game's mods folder

The desktop app and the companion are separate. The installer or portable ZIP gives you the
tracker interface; the companion is what passively reports discoveries while
the game is running. The companion does not edit saves or replace game values.

## Install the desktop app

1. Run `MistriaTracker-0.1.3-setup.exe`. Approve the normal Windows UAC prompt
   if the game is installed in a protected folder such as `Program Files`.
2. In the installer, optionally tick **Live tracking companion (MOMI, recommended)**. Select
   the game's `mods` folder when prompted. The installer copies only the
   companion files and never reads or modifies a save.
3. If the installer is blocked, use the portable ZIP instead: extract it and
   run `MistriaTracker.exe`.
4. Launch **Mistria Tracker**.
5. Keep only one tracker window open. A second launch is refused and focuses the
   existing window.

## Install the companion with MOMI

1. If you skipped the companion during installation, download and extract
   `MistriaTracker-Companion-0.1.3.zip`. From source,
   create it with `pnpm package:companion`.
2. Copy the extracted `MistriaTrackerCompanion` folder directly into the
   Fields of Mistria `mods` folder. Do not nest it inside another folder.
3. Confirm that the installed folder contains `manifest.json` and
   `gml/MistriaTrackerCompanion.gml` directly under the mod folder. Do not nest
   it inside a second `MistriaTracker` folder.
4. Start Fields of Mistria through MOMI.

## First-run checklist

1. Open the tracker Settings page and run the read-only readiness check.
2. Wait for the message that the catalog is approved and the companion log is
   ready. If the game build is unknown, live tracking stays paused safely.
3. Start with a newly created disposable profile. Do not test first with an
   important save.
4. Obtain one harmless item, return to the tracker, and confirm the discovery
   appears. The tracker reads the companion's isolated log; it never writes to a
   Fields of Mistria save.

## If the tracker finds nothing

Installing the desktop tracker alone is not enough for live tracking. The
desktop app does not scan arbitrary save files or choose a profile automatically
because doing so could read the wrong save. It waits for the companion's
isolated event log.

Check all of the following:

1. MOMI/Mods of Mistria 0.14.1 or newer is installed.
2. Fields of Mistria was started through MOMI after the companion was copied.
3. The companion has this exact layout:

   ```text
   Fields of Mistria/
   └─ mods/
      └─ MistriaTrackerCompanion/
         ├─ manifest.json
         └─ gml/
            └─ MistriaTrackerCompanion.gml
   ```

   Do not leave an extra nested folder such as
   `mods/MistriaTrackerCompanion/MistriaTrackerCompanion/`.
4. The tracker Settings page reports that the catalog is approved and the
   companion log is ready.
5. A disposable character is loaded and one new item or event is performed.

If the readiness check remains paused, close the game, correct the companion
layout, start the game through MOMI again, and rerun the check. Never delete,
move, or overwrite an important save to troubleshoot the tracker.

## Spoiler and language settings

Spoiler-free mode is the default. Unknown entries remain hidden until observed.
Use Settings to switch to “Show everything” or choose English/French.

## If live tracking is paused

- Make sure MOMI is installed and the companion is enabled and deployed.
- Make sure the game is running the approved version.
- Restart the game after changing the deployed mod.
- Re-run the read-only readiness check.

Never delete or overwrite a save to troubleshoot the tracker. Keep a manual copy
of important saves before testing any mod.
