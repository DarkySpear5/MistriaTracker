Unicode true
!include "Sections.nsh"

Name "Mistria Tracker"
Icon "..\src-tauri\icons\icon.ico"
UninstallIcon "..\src-tauri\icons\icon.ico"
OutFile "installer\\MistriaTracker-0.1.4-setup.exe"
InstallDir "$LOCALAPPDATA\\Mistria Tracker"
; Elevation is needed only when Steam is installed under Program Files and the
; optional companion is copied into its verified game mods folder.
RequestExecutionLevel highest
ShowInstDetails show
ShowUnInstDetails show

Var GameDirectory

; Check only known Steam locations. This never scans folders recursively and a
; candidate is accepted only if it contains Fields of Mistria's assets.zip.
Function FindGameDirectory
  StrCpy $GameDirectory ""
  ReadRegStr $R0 HKCU "Software\\Valve\\Steam" "SteamPath"
  IfFileExists "$R0\\steamapps\\common\\Fields of Mistria\\assets.zip" steam_root_found

  ; Check the common Steam and SteamLibrary folders on mounted C: through Z:
  ; drives. Unusual library names use the one-time fallback picker below.
  StrCpy $R1 67
  find_drive:
  IntFmt $R2 "%c" $R1
  StrCpy $R0 "$R2:\\SteamLibrary\\steamapps\\common\\Fields of Mistria"
  IfFileExists "$R0\\assets.zip" drive_found
  StrCpy $R0 "$R2:\\Steam\\steamapps\\common\\Fields of Mistria"
  IfFileExists "$R0\\assets.zip" drive_found
  StrCpy $R0 "$R2:\\Program Files (x86)\\Steam\\steamapps\\common\\Fields of Mistria"
  IfFileExists "$R0\\assets.zip" drive_found
  StrCpy $R0 "$R2:\\Program Files\\Steam\\steamapps\\common\\Fields of Mistria"
  IfFileExists "$R0\\assets.zip" drive_found
  IntOp $R1 $R1 + 1
  IntCmp $R1 91 find_drive find_done find_done

  steam_root_found:
  StrCpy $GameDirectory "$R0\\steamapps\\common\\Fields of Mistria"
  Return

  drive_found:
  StrCpy $GameDirectory "$R0"
  find_done:
FunctionEnd

Page components
Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "Mistria Tracker"
  SetOutPath "$INSTDIR"
  File "..\\src-tauri\\target\\release\\mistria-tracker.exe"
  WriteUninstaller "$INSTDIR\\Uninstall Mistria Tracker.exe"
  CreateDirectory "$SMPROGRAMS\\Mistria Tracker"
  CreateShortcut "$SMPROGRAMS\\Mistria Tracker\\Mistria Tracker.lnk" "$INSTDIR\\mistria-tracker.exe"
  CreateShortcut "$SMPROGRAMS\\Mistria Tracker\\Uninstall Mistria Tracker.lnk" "$INSTDIR\\Uninstall Mistria Tracker.exe"
SectionEnd

Section /o "Add a desktop shortcut" SecDesktopShortcut
  CreateShortcut "$DESKTOP\\Mistria Tracker.lnk" "$INSTDIR\\mistria-tracker.exe"
SectionEnd

Section /o "Live tracking companion (AIM/MOMI, recommended)" SecCompanion
  StrCmp $GameDirectory "" choose_game_folder game_directory_ready

  choose_game_folder:
  nsDialogs::SelectFolderDialog "Fields of Mistria was not found automatically. Choose the game folder that contains assets.zip." "$PROGRAMFILES"
  Pop $GameDirectory
  StrCmp $GameDirectory "error" companion_skipped
  StrCmp $GameDirectory "" companion_skipped

  game_directory_ready:
  IfFileExists "$GameDirectory\\assets.zip" companion_install companion_invalid_folder

  companion_invalid_folder:
  MessageBox MB_ICONEXCLAMATION "The selected folder does not contain Fields of Mistria's assets.zip. The tracker was installed, but the companion was skipped. Run setup again after choosing the game's folder."
  Goto companion_skipped

  companion_install:
  ; Remove only the two known files from the old 0.1.0 folder. Never recurse
  ; through a user-selected folder or remove unrelated mod files.
  Delete "$GameDirectory\\mods\\mistria_tracker_companion\\manifest.json"
  Delete "$GameDirectory\\mods\\mistria_tracker_companion\\gml\\MistriaTrackerCompanion.gml"
  RMDir "$GameDirectory\\mods\\mistria_tracker_companion\\gml"
  RMDir "$GameDirectory\\mods\\mistria_tracker_companion"
  SetOutPath "$GameDirectory\\mods\\MistriaTrackerCompanion"
  File /r "..\companion\mistria_tracker_companion\*.*"

  companion_skipped:
SectionEnd

Function .onInit
  Call FindGameDirectory
  SectionSetFlags ${SecDesktopShortcut} ${SF_SELECTED}
  StrCmp $GameDirectory "" game_not_found
  SectionSetFlags ${SecCompanion} ${SF_SELECTED}
  game_not_found:
FunctionEnd

Section "Uninstall"
  Delete "$DESKTOP\\Mistria Tracker.lnk"
  Delete "$SMPROGRAMS\\Mistria Tracker\\Mistria Tracker.lnk"
  Delete "$SMPROGRAMS\\Mistria Tracker\\Uninstall Mistria Tracker.lnk"
  RMDir "$SMPROGRAMS\\Mistria Tracker"
  Delete "$INSTDIR\\mistria-tracker.exe"
  Delete "$INSTDIR\\Uninstall Mistria Tracker.exe"
  RMDir "$INSTDIR"
  ; Tracker data and Fields of Mistria saves are deliberately never removed.
SectionEnd
