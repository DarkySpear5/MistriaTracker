Unicode true
!include "Sections.nsh"
!include "StrFunc.nsh"

${StrRep}

Name "Mistria Tracker"
Icon "..\src-tauri\icons\icon.ico"
UninstallIcon "..\src-tauri\icons\icon.ico"
!ifndef OUTPUT_FILE
!define OUTPUT_FILE "installer\\MistriaTracker-0.1.5-setup.exe"
!endif
OutFile "${OUTPUT_FILE}"
InstallDir "$LOCALAPPDATA\\Mistria Tracker"
; Elevation is needed only when Steam is installed under Program Files and the
; optional companion is copied into its verified game mods folder.
; The optional companion is installed into Steam's protected Program Files tree.
; Request elevation up front so the installer can copy it reliably instead of
; installing the desktop app and then failing part-way through.
RequestExecutionLevel admin
ShowInstDetails show
ShowUnInstDetails show

Var GameDirectory
Var CompanionTarget

; Check only known Steam locations. This never scans folders recursively and a
; candidate is accepted only if it contains Fields of Mistria's assets.zip.
Function FindGameDirectory
  StrCpy $GameDirectory ""
  ReadRegStr $R0 HKCU "Software\\Valve\\Steam" "SteamPath"
  ; Steam commonly stores this value with forward slashes. Normalize it before
  ; appending Windows paths so NSIS receives one canonical destination format.
  ${StrRep} $R0 $R0 "/" "\\"
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
  CreateShortcut "$SMPROGRAMS\\Mistria Tracker\\Mistria Tracker.lnk" "$INSTDIR\\mistria-tracker.exe" "" "$INSTDIR\\mistria-tracker.exe" 0
  CreateShortcut "$SMPROGRAMS\\Mistria Tracker\\Uninstall Mistria Tracker.lnk" "$INSTDIR\\Uninstall Mistria Tracker.exe"
SectionEnd

Section /o "Add a desktop shortcut" SecDesktopShortcut
  CreateShortcut "$DESKTOP\\Mistria Tracker.lnk" "$INSTDIR\\mistria-tracker.exe" "" "$INSTDIR\\mistria-tracker.exe" 0
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
  StrCpy $CompanionTarget "$GameDirectory\\mods\\MistriaTrackerCompanion"
  ; Prepare the exact destination without touching unrelated mod files.
  ClearErrors
  CreateDirectory "$CompanionTarget"

  ; Extract the two required runtime files directly into the validated game
  ; folder. The installer is elevated, so this also works for Steam installs
  ; under Program Files without a second copy operation or path translation.
  SetOverwrite on
  SetOutPath "$CompanionTarget"
  File "..\\companion\\mistria_tracker_companion\\manifest.json"
  SetOutPath "$CompanionTarget\\gml"
  File "..\\companion\\mistria_tracker_companion\\gml\\MistriaTrackerCompanion.gml"
  IfFileExists "$CompanionTarget\\manifest.json" companion_manifest_ok companion_failed
  companion_manifest_ok:
  IfFileExists "$CompanionTarget\\gml\\MistriaTrackerCompanion.gml" companion_installed companion_failed

  companion_installed:
  MessageBox MB_ICONINFORMATION "The live-tracking companion files were installed.$\r$\n$\r$\nOpen AIM or MOMI and apply/rebuild your mods once before starting Fields of Mistria."
  Goto companion_skipped

  companion_failed:
  MessageBox MB_ICONEXCLAMATION "The Tracker app was installed, but the optional companion could not be copied to:$\r$\n$CompanionTarget$\r$\n$\r$\nCheck that Fields of Mistria is closed and that you have permission to write to this folder, then run setup again as administrator."

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
