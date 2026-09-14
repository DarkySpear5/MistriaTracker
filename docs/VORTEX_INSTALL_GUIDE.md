# Mistria Tracker — Vortex installation guide

## What is required

- Windows 10 or Windows 11 (64-bit)
- Fields of Mistria installed and updated
- MOMI/Mods of Mistria 0.14.1 or newer
- The Mistria Tracker desktop app installer
- The companion mod enabled in Vortex

The desktop app and the companion are separate. The installer gives you the
tracker interface; the companion is what passively reports discoveries while
the game is running. The companion does not edit saves or replace game values.

## Install the desktop app

1. Run `MistriaTracker-0.1.0-setup.exe` as a normal user. Administrator access
   should not be required.
2. Launch **Mistria Tracker** from the Start menu.
3. Keep only one tracker window open. A second launch is refused and focuses the
   existing window.

## Install the companion with Vortex

1. In the GitHub repository, download the repository ZIP or the companion folder
   from `companion/mistria_tracker_companion`.
2. In Vortex, install the mod from the downloaded ZIP, then enable and deploy it.
3. Confirm that the deployed folder contains the companion `manifest.json` and
   `gml/MistriaTrackerCompanion.gml` directly under the mod folder. Do not nest
   it inside a second `MistriaTracker` folder.
4. Start Fields of Mistria through Vortex/MOMI.

## First-run checklist

1. Open the tracker Settings page and run the read-only readiness check.
2. Wait for the message that the catalog is approved and the companion log is
   ready. If the game build is unknown, live tracking stays paused safely.
3. Start with a newly created disposable profile. Do not test first with an
   important save.
4. Obtain one harmless item, return to the tracker, and confirm the discovery
   appears. The tracker reads the companion's isolated log; it never writes to a
   Fields of Mistria save.

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
