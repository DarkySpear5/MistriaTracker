# Mistria Tracker — MOMI installation guide

## What is required

- Windows 10 or Windows 11 (64-bit)
- Fields of Mistria installed and updated
- The Mistria Tracker desktop app installer
- MOMI/Mods of Mistria 0.14.1 or newer, only for live tracking while playing

The desktop app and the companion are separate. The companion is only needed
to report discoveries while the game is running. It does not edit saves or
replace game values.

## Install the desktop app

1. Run `MistriaTracker-0.1.3-setup.exe` and install it normally.
2. To use live tracking, tick **Live tracking companion (MOMI, recommended)**
   on the Components page. Select the game's `mods` folder when prompted.
3. Launch **Mistria Tracker**. It automatically loads existing discoveries
   when it opens.
4. Keep only one tracker window open. A second launch focuses the existing
   window.

## Install the companion with MOMI

1. If you skipped the companion during installation, run the installer again
   and tick **Live tracking companion (MOMI, recommended)**.
2. Select the Fields of Mistria `mods` folder when prompted.
3. Start Fields of Mistria through MOMI.

## First-run checklist

1. Start Fields of Mistria through MOMI, then open Mistria Tracker.
2. The tracker automatically checks whether live tracking is available. There
   is nothing to refresh or approve manually.
3. When the companion is detected, new discoveries appear in the tracker while
   you play. The tracker reads the companion's isolated log and never writes to
   a Fields of Mistria save.

## If the tracker finds nothing

Existing discoveries load automatically when the tracker opens. If new
discoveries are not appearing while you play:

1. Make sure MOMI is installed, then start Fields of Mistria through MOMI.
2. Close and reopen both the game and Mistria Tracker.
3. In Mistria Tracker Settings, look at the live-tracking status. It is
   informational only; there is nothing to refresh or approve manually.
4. If the companion was skipped during setup, run the installer again and tick
   **Live tracking companion (MOMI, recommended)**.

Never delete, move, or overwrite a save to troubleshoot the tracker.

## Spoiler and language settings

Spoiler-free mode is the default. Unknown entries remain hidden until observed.
Use Settings to switch to “Show everything” or choose English/French.

## If live tracking is paused

- Make sure MOMI is installed and the companion is enabled and deployed.
- Make sure the game is running the approved version.
- Restart the game after changing the deployed mod.
- Open Settings to view the live-tracking status.

Never delete or overwrite a save to troubleshoot the tracker. Keep a manual copy
of important saves before testing any mod.
