Unicode true
!include "Sections.nsh"

Name "Mistria Tracker"
Icon "..\src-tauri\icons\icon.ico"
UninstallIcon "..\src-tauri\icons\icon.ico"
!ifndef OUTPUT_FILE
!define OUTPUT_FILE "installer\\MistriaTracker-0.1.6-setup.exe"
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
  ; Reuse the app's bounded, tested Steam libraryfolders.vdf discovery instead
  ; of maintaining a second hard-coded installer search.
  StrCpy $GameDirectory ""
  ClearErrors
  nsExec::ExecToStack '"$INSTDIR\\mistria-tracker.exe" --print-game-directory'
  Pop $R0
  Pop $GameDirectory
  StrCmp $R0 "0" game_directory_ready
  StrCpy $GameDirectory ""
  Goto choose_game_folder

  choose_game_folder:
  nsDialogs::SelectFolderDialog "Fields of Mistria was not found automatically. Choose the Fields of Mistria game folder." "$PROGRAMFILES"
  Pop $GameDirectory
  StrCmp $GameDirectory "error" companion_skipped
  StrCmp $GameDirectory "" companion_skipped

  game_directory_ready:
  IfFileExists "$GameDirectory\\Maybe.toml" game_signature_found companion_invalid_folder
  game_signature_found:
  IfFileExists "$GameDirectory\\assets.zip" companion_install companion_invalid_folder

  companion_invalid_folder:
  MessageBox MB_ICONEXCLAMATION "The selected folder is not a valid Fields of Mistria installation. Choose the main game folder containing Maybe.toml. The Tracker app was installed, but the Companion was skipped."
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
  MessageBox MB_ICONINFORMATION|MB_YESNO "The live-tracking Companion files were copied successfully.$\r$\n$\r$\nThey are not active yet. Open AIM or run MOMI and click Install/Apply once before launching Fields of Mistria. Repeat this after every game update.$\r$\n$\r$\nMOMI is a portable tool; it does not stay installed or run with the game.$\r$\n$\r$\nOpen the official MOMI download page now?" IDYES open_momi IDNO companion_skipped
  open_momi:
  ExecShell "open" "https://github.com/Garethp/Mods-of-Mistria-Installer/releases/latest"
  Goto companion_skipped

  companion_failed:
  MessageBox MB_ICONEXCLAMATION "The Tracker app was installed, but the optional companion could not be copied to:$\r$\n$CompanionTarget$\r$\n$\r$\nCheck that Fields of Mistria is closed and that you have permission to write to this folder, then run setup again as administrator."

  companion_skipped:
SectionEnd

Function .onInit
  SectionSetFlags ${SecDesktopShortcut} ${SF_SELECTED}
  SectionSetFlags ${SecCompanion} ${SF_SELECTED}
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
