Unicode true
Name "Mistria Tracker"
Icon "..\src-tauri\icons\icon.ico"
UninstallIcon "..\src-tauri\icons\icon.ico"
OutFile "installer\\MistriaTracker-0.1.0-setup.exe"
InstallDir "$LOCALAPPDATA\\Mistria Tracker"
RequestExecutionLevel user
ShowInstDetails show
ShowUnInstDetails show

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

Section "Uninstall"
  Delete "$SMPROGRAMS\\Mistria Tracker\\Mistria Tracker.lnk"
  Delete "$SMPROGRAMS\\Mistria Tracker\\Uninstall Mistria Tracker.lnk"
  RMDir "$SMPROGRAMS\\Mistria Tracker"
  Delete "$INSTDIR\\mistria-tracker.exe"
  Delete "$INSTDIR\\Uninstall Mistria Tracker.exe"
  RMDir "$INSTDIR"
  ; Tracker data and Fields of Mistria saves are deliberately never removed.
SectionEnd
