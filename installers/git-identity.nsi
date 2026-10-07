; Per-user Windows installer for git-identity (no admin rights required).
;
;   makensis /DVERSION=1.1.0 /DARCH=x86_64 /DBINARY=..\target\release\git-identity.exe ^
;            /DOUTFILE=..\dist\git-identity-1.1.0-x86_64-setup.exe installers\git-identity.nsi

!ifndef VERSION
  !error "Pass /DVERSION=x.y.z"
!endif
!ifndef ARCH
  !define ARCH "x86_64"
!endif
!ifndef BINARY
  !define BINARY "..\target\release\git-identity.exe"
!endif
!ifndef OUTFILE
  !ifdef NSIS_WIN32_MAKENSIS
    !system 'if not exist "..\dist" mkdir "..\dist"'
  !else
    !system 'mkdir -p ../dist'
  !endif
  !define OUTFILE "..\dist\git-identity-${VERSION}-${ARCH}-setup.exe"
!endif

!define APP      "git-identity"
!define UNINST   "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP}"
; Edits HKCU\Environment\Path through the registry API so REG_EXPAND_SZ entries
; such as %USERPROFILE%\... are preserved (setx / [Environment] would flatten them).
; $e is the user PATH minus our own entry, with everything else left untouched
; (empty entries included) so an install followed by an uninstall restores it exactly.
!define PS_PATH  "powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command \
  $$k=[Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('Environment'); \
  $$p=[string]$$k.GetValue('Path','',[Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames); \
  $$d='$INSTDIR'; $$e=@(); if ($$p) { $$e=@($$p -split ';' | Where-Object { $$_.TrimEnd('\') -ne $$d }) };"

Unicode true
SetCompressor /SOLID lzma
RequestExecutionLevel user

Name "${APP} ${VERSION}"
OutFile "${OUTFILE}"
InstallDir "$LOCALAPPDATA\Programs\${APP}"
InstallDirRegKey HKCU "${UNINST}" "InstallLocation"

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "${APP}"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "FileDescription" "${APP} installer"
VIAddVersionKey "LegalCopyright" "MIT License"

!include "MUI2.nsh"

!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_TITLE "${APP} is installed"
!define MUI_FINISHPAGE_TEXT "${APP} was added to your user PATH.$\r$\n$\r$\nOpen a new terminal, then run:$\r$\n$\r$\n    git identity init$\r$\n$\r$\nto create your identities config."

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "French"

Section "Install"
  SetOutPath "$INSTDIR"
  File "/oname=git-identity.exe" "${BINARY}"
  File "/oname=LICENSE.txt" "..\LICENSE"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  DetailPrint "Adding $INSTDIR to the user PATH"
  nsExec::ExecToLog `${PS_PATH} $$k.SetValue('Path', (($$e + $$d) -join ';'), 'ExpandString')`
  Pop $0
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000

  WriteRegStr   HKCU "${UNINST}" "DisplayName" "${APP}"
  WriteRegStr   HKCU "${UNINST}" "DisplayVersion" "${VERSION}"
  WriteRegStr   HKCU "${UNINST}" "Publisher" "Dxsk"
  WriteRegStr   HKCU "${UNINST}" "URLInfoAbout" "https://github.com/Dxsk/git-identity-manager"
  WriteRegStr   HKCU "${UNINST}" "InstallLocation" "$INSTDIR"
  WriteRegStr   HKCU "${UNINST}" "DisplayIcon" "$INSTDIR\git-identity.exe"
  WriteRegStr   HKCU "${UNINST}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr   HKCU "${UNINST}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegDWORD HKCU "${UNINST}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINST}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  nsExec::ExecToLog `${PS_PATH} $$k.SetValue('Path', ($$e -join ';'), 'ExpandString')`
  Pop $0
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000

  Delete "$INSTDIR\git-identity.exe"
  Delete "$INSTDIR\LICENSE.txt"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "${UNINST}"
SectionEnd
