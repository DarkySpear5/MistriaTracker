Unicode true
Name "Mistria Tracker"
Icon "..\src-tauri\icons\icon.ico"
UninstallIcon "..\src-tauri\icons\icon.ico"
OutFile "installer\\MistriaTracker-0.1.2-setup.exe"
InstallDir "$LOCALAPPDATA\\Mistria Tracker"
RequestExecutionLevel highest
ShowInstDetails show
ShowUnInstDetails show

Var CompanionModsPath

Function .onInit
  ReadRegStr $CompanionModsPath HKCU "Software\\Valve\\Steam" "SteamPath"
  StrCmp $CompanionModsPath "" 0 +2
  StrCpy $CompanionModsPath "$PROGRAMFILES\\Steam"
  StrCpy $CompanionModsPath "$CompanionModsPath\\steamapps\\common\\Fields of Mistria\\mods"
FunctionEnd

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

Section /o "Live tracking companion (MOMI)"
  nsDialogs::SelectFolderDialog "Choose your Fields of Mistria mods folder" "$CompanionModsPath"
  Pop $CompanionModsPath
  StrCmp $CompanionModsPath "error" companion_skipped
  StrCmp $CompanionModsPath "" companion_skipped
  SetOutPath "$CompanionModsPath\\MistriaTrackerCompanion"
  File /r "..\\companion\\mistria_tracker_companion\\*.*"
  companion_skipped:
SectionEnd

Section "Uninstall"
  Delete "$SMPROGRAMS\\Mistria Tracker\\Mistria Tracker.lnk"
  Delete "$SMPROGRAMS\\Mistria Tracker\\Uninstall Mistria Tracker.lnk"
  RMDir "$SMPROGRAMS\\Mistria Tracker"
  Delete "$INSTDIR\\mistria-tracker.exe"
  Delete "$INSTDIR\\Uninstall Mistria Tracker.exe"
  RMDir "$INSTDIR"
  ; Tracker data and Fields of Mistria saves are deliberately never removed.
SectionEnd
