# Mistria Tracker — AIM and MOMI installation guide

## What is required

- Windows 10 or Windows 11 (64-bit)
- Fields of Mistria installed and updated
- The Mistria Tracker desktop app installer
- AIM or MOMI/Mods of Mistria with compatible MMAPI hooks, only for live
  tracking while playing

The desktop app and the Companion are separate. Tracker's installer copies the
Companion files, but AIM or MOMI must apply those files to the game once before
live tracking can work. The Companion does not edit saves or replace game
values.

## Install the desktop app

1. Download the latest Windows installer from the
   [official Mistria Tracker Releases page](https://github.com/DarkySpear5/MistriaTracker/releases/latest).
2. Close Fields of Mistria, run `MistriaTracker-0.1.6-setup.exe`, and install
   it normally. The
   **Add a desktop shortcut** choice is selected by default; untick it if you
   do not want one.
3. To use live tracking, leave **Live tracking companion (AIM/MOMI,
   recommended)** selected. Setup finds normal Steam installs automatically.
   It also reads Steam's `libraryfolders.vdf` for custom libraries. If it
   cannot find the game, select the main *Fields of Mistria* folder containing
   `Maybe.toml`—not `assets.zip` and not the `mods` folder.
4. When setup finishes copying the Companion, complete one of the AIM/MOMI
   sections below before launching the game.

## Apply the Companion with AIM

1. Open AIM after Tracker's installer has copied the Companion.
2. Confirm that Mistria Tracker Companion is enabled in the active profile.
3. Apply or rebuild the profile.
4. Close AIM when it completes. Launch Fields of Mistria normally through
   Steam.

## Apply the Companion with MOMI

MOMI is a portable program, not a traditional installed application.

1. Download the latest Windows `ModsOfMistriaInstaller.exe` from the
   [official MOMI Releases page](https://github.com/Garethp/Mods-of-Mistria-Installer/releases/latest).
   Do not download the OSX or Linux archive on Windows.
2. Close Fields of Mistria and run the downloaded MOMI executable.
3. MOMI should detect the game and list **Mistria Tracker Companion**. Keep it
   checked and click **Install**.
4. After MOMI finishes, close it and launch Fields of Mistria normally through
   Steam. MOMI does not need to remain open.

If MOMI cannot locate the game, place its executable temporarily in the main
Fields of Mistria folder beside `Maybe.toml`, then run it again.

## First-run checklist

1. Launch Fields of Mistria normally, then open Mistria Tracker.
2. The Tracker automatically checks whether live tracking is available. There
   is nothing to refresh or approve manually.
3. After loading a save, change rooms once. This creates the companion log,
   identifies the exact active save, imports its approved existing data from a
   temporary read-only copy, and begins live tracking.
4. New discoveries then appear automatically while you play. The tracker reads
   the companion's isolated log and never writes to a Fields of Mistria save.

## If the tracker finds nothing

If existing discoveries do not appear after choosing a save file, or if new
discoveries are not appearing while you play:

1. Confirm that
   `Fields of Mistria\mods\MistriaTrackerCompanion\manifest.json` exists.
2. Remember that this folder only proves the Companion was copied. Open AIM
   and apply/rebuild, or run MOMI and click **Install**.
3. Close and reopen both the game and Mistria Tracker. Load the save and change
   rooms once.
4. In Mistria Tracker Settings, choose **Load save file** for existing data,
   then look at the live-tracking status for new events. It is informational
   only; there is nothing to approve manually.
5. If the Companion was skipped during setup, run the installer again and
   select **Live tracking companion (AIM/MOMI, recommended)**.

When live tracking is active, the Companion log appears at:

`%LOCALAPPDATA%\FieldsOfMistria\mod_data\mistria_tracker_companion\logs\mistria_tracker_companion.log`

If manual import reports an older unsupported save version, open that
character in the current Fields of Mistria version, save and close the game,
then select the updated `.sav` file. Tracker never changes the original.

Never delete, move, or overwrite a save to troubleshoot the tracker.

## Spoiler and language settings

Spoiler-free mode is the default. Unknown entries remain hidden until observed.
Use Settings to switch to “Show everything” or choose English/French.

## If live tracking is paused

- Make sure the Companion is enabled and applied through AIM or MOMI.
- Make sure the game is running the approved version.
- Run AIM's apply/rebuild action or MOMI's **Install** button again after every
  game update and after replacing Companion files.
- Restart the game after applying the Companion.
- Open Settings to view the live-tracking status.

Never delete or overwrite a save to troubleshoot the tracker. Keep a manual copy
of important saves before testing any mod.
