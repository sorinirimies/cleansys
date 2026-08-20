; CleanSys Windows Installer
; Built with NSIS (Nullsoft Scriptable Install System)
;
; NOTE: this script is compiled from a versioned copy at
; dist\installer_versioned.nsi (see scripts/ci/package_windows.sh), not from
; its own committed location (packaging\windows\installer.nsi). NSIS resolves
; every relative path used by File/LicenseData/Icon-style commands relative
; to the *compiled script's own directory*, not the process's working
; directory or the script's original location. Left unhandled, that means
; "target\...\release\cleansys-gui.exe" would be looked up under
; dist\target\...\, and "LICENSE" under dist\LICENSE — neither of which
; exist, producing errors like:
;   LicenseData: open failed "LICENSE"
;   Error while loading icon from "packaging\windows\cleansys.ico": can't open file
; `!cd` re-anchors that resolution to the actual repository root
; (substituted below by package_windows.sh via @REPO_ROOT_ABS@), so every
; relative path elsewhere in this file behaves exactly as if the script
; were compiled in place at the repo root.
!cd "@REPO_ROOT_ABS@"

!define PRODUCT_NAME "CleanSys"
!define PRODUCT_VERSION "@VERSION@"
!define PRODUCT_PUBLISHER "Sorin Albu-Irimies"
!define PRODUCT_URL "https://github.com/sorinirimies/cleansys"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "dist\cleansys-${PRODUCT_VERSION}-windows-x86_64-setup.exe"
InstallDir "$PROGRAMFILES64\CleanSys"
InstallDirRegKey HKLM "${PRODUCT_UNINST_KEY}" "InstallLocation"
RequestExecutionLevel admin
SetCompressor /SOLID lzma

!include "MUI2.nsh"

; UI
!define MUI_ABORTWARNING
!define MUI_ICON "packaging\windows\cleansys.ico"
!define MUI_UNICON "packaging\windows\cleansys.ico"

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

; Components
Section "CleanSys GUI" SecGUI
    SectionIn RO
    SetOutPath "$INSTDIR"
    File "target\x86_64-pc-windows-msvc\release\cleansys-gui.exe"
    CreateDirectory "$SMPROGRAMS\CleanSys"
    CreateShortcut "$SMPROGRAMS\CleanSys\CleanSys.lnk" "$INSTDIR\cleansys-gui.exe"
    CreateShortcut "$DESKTOP\CleanSys.lnk" "$INSTDIR\cleansys-gui.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\App Paths\cleansys-gui.exe" "" "$INSTDIR\cleansys-gui.exe"
SectionEnd

Section "CleanSys TUI/CLI" SecTUI
    SetOutPath "$INSTDIR"
    File "target\x86_64-pc-windows-msvc\release\cleansys.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\App Paths\cleansys.exe" "" "$INSTDIR\cleansys.exe"
SectionEnd

; PATH key in the system environment registry
!define PATH_KEY "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"

Section "Add to PATH" SecPATH
    ; Append install directory to the system PATH via registry — no plugin required.
    ReadRegStr $0 HKLM "${PATH_KEY}" "Path"
    WriteRegExpandStr HKLM "${PATH_KEY}" "Path" "$0;$INSTDIR"
    ; Broadcast the change so running processes pick it up immediately.
    SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
SectionEnd

; Descriptions
LangString DESC_SecGUI ${LANG_ENGLISH} "CleanSys desktop GUI — mouse-driven system cleaner"
LangString DESC_SecTUI ${LANG_ENGLISH} "CleanSys TUI/CLI — keyboard-driven terminal system cleaner"
LangString DESC_SecPATH ${LANG_ENGLISH} "Add CleanSys to the system PATH"

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
    !insertmacro MUI_DESCRIPTION_TEXT ${SecGUI} $(DESC_SecGUI)
    !insertmacro MUI_DESCRIPTION_TEXT ${SecTUI} $(DESC_SecTUI)
    !insertmacro MUI_DESCRIPTION_TEXT ${SecPATH} $(DESC_SecPATH)
!insertmacro MUI_FUNCTION_DESCRIPTION_END

Section -PostInstall
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayName" "${PRODUCT_NAME} ${PRODUCT_VERSION}"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "InstallLocation" "$INSTDIR"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_URL}"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
    WriteRegDWORD HKLM "${PRODUCT_UNINST_KEY}" "NoModify" 1
    WriteRegDWORD HKLM "${PRODUCT_UNINST_KEY}" "NoRepair" 1
    WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section Uninstall
    Delete "$INSTDIR\cleansys-gui.exe"
    Delete "$INSTDIR\cleansys.exe"
    Delete "$INSTDIR\uninstall.exe"
    Delete "$SMPROGRAMS\CleanSys\CleanSys.lnk"
    Delete "$DESKTOP\CleanSys.lnk"
    RMDir "$SMPROGRAMS\CleanSys"
    RMDir "$INSTDIR"
    ; Remove install directory from the system PATH via registry.
    ReadRegStr $0 HKLM "${PATH_KEY}" "Path"
    ; Build cleaned PATH by removing our entry (exact match).
    ; Uses PowerShell for reliable semicolon-delimited string manipulation.
    ExecWait 'powershell -NoProfile -Command "\
        $p = [Environment]::GetEnvironmentVariable(\"Path\",\"Machine\"); \
        $clean = ($p -split \";\") | Where-Object { $_ -ne \"$INSTDIR\" }; \
        [Environment]::SetEnvironmentVariable(\"Path\", ($clean -join \";\"), \"Machine\")"'
    DeleteRegKey HKLM "${PRODUCT_UNINST_KEY}"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\App Paths\cleansys-gui.exe"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\App Paths\cleansys.exe"
SectionEnd
