# Mistria Tracker — AIM and MOMI installation guide

## What is required

- Windows 10 or Windows 11 (64-bit)
- Fields of Mistria installed and updated
- The Mistria Tracker desktop app installer
- AIM or MOMI/Mods of Mistria with compatible MMAPI hooks, only for live
  tracking while playing

The desktop app and the companion are separate. The companion is only needed
to report discoveries while the game is running. It does not edit saves or
replace game values.

## Install the desktop app

1. Run `MistriaTracker-0.1.5-setup.exe` and install it normally. The
   **Add a desktop shortcut** choice is selected by default; untick it if you
   do not want one.
2. To use live tracking, leave **Live tracking companion (AIM/MOMI,
   recommended)** selected. Setup finds normal Steam installs automatically.
   If it cannot, select the *game folder containing `assets.zip`*—not the
   `mods` folder.
3. Launch **Mistria Tracker**, open Settings, and choose **Load save file**.
   Select a `.sav` file. Tracker reads a temporary copy and never changes the
   original save. If the save's game version is not approved for import yet,
   Tracker reports that without changing the save or creating tracker
   discoveries.
4. Keep only one tracker window open. A second launch focuses the existing
   window.

## Install the companion with AIM or MOMI

1. If you skipped the companion during installation, run the installer again
   and select **Live tracking companion (AIM/MOMI, recommended)**.
2. Setup installs the companion under the verified game folder's `mods`
   directory. If automatic detection cannot find the game, choose the game
   folder containing `assets.zip`.
3. Start Fields of Mistria through AIM or MOMI.

## First-run checklist

1. Start Fields of Mistria through MOMI, then open Mistria Tracker.
2. The tracker automatically checks whether live tracking is available. There
   is nothing to refresh or approve manually.
3. When the companion is detected, new discoveries appear in the tracker while
   you play. The tracker reads the companion's isolated log and never writes to
   a Fields of Mistria save.
4. After loading a save for the first time, change rooms once. This creates the
   companion log and begins live tracking.

## If the tracker finds nothing

If existing discoveries do not appear after choosing a save file, or if new
discoveries are not appearing while you play:

1. Make sure MOMI is installed, then start Fields of Mistria through MOMI.
2. Close and reopen both the game and Mistria Tracker.
3. In Mistria Tracker Settings, choose **Load save file** for existing data,
   then look at the live-tracking status for new events. It is informational
   only; there is nothing to approve manually.
4. If the companion was skipped during setup, run the installer again and
   select **Live tracking companion (AIM/MOMI, recommended)**.

Never delete, move, or overwrite a save to troubleshoot the tracker.

## Spoiler and language settings

Spoiler-free mode is the default. Unknown entries remain hidden until observed.
Use Settings to switch to “Show everything” or choose English/French.

## If live tracking is paused

- Make sure AIM or MOMI is installed and the companion is enabled and deployed.
- Make sure the game is running the approved version.
- Restart the game after changing the deployed mod.
- Open Settings to view the live-tracking status.

Never delete or overwrite a save to troubleshoot the tracker. Keep a manual copy
of important saves before testing any mod.
