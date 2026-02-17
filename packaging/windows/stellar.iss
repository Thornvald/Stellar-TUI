#ifndef StellarVersion
#define StellarVersion "0.91.0"
#endif

[Setup]
AppId={{D5EED8B6-E8B8-4B40-B2BC-956D41A4C509}
AppName=Stellar
AppVersion={#StellarVersion}
AppPublisher=Stellar
DefaultDirName={autopf}\Stellar
DefaultGroupName=Stellar
OutputDir=dist
OutputBaseFilename=stellar-setup-{#StellarVersion}
Compression=lzma
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SetupIconFile=..\..\icon.ico
UninstallDisplayIcon={app}\stellar.exe
ChangesEnvironment=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "..\..\target\release\stellar.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\completed.wav"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\fail.wav"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Stellar"; Filename: "{app}\stellar.exe"
Name: "{group}\Uninstall Stellar"; Filename: "{uninstallexe}"

[Run]
Filename: "{app}\stellar.exe"; Description: "Launch Stellar"; Flags: nowait postinstall skipifsilent

[Registry]
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Check: NeedsAddPath(ExpandConstant('{app}'))

[Code]
function NeedsAddPath(Param: string): Boolean;
var
  Paths: string;
begin
  Result := True;
  if RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', Paths) then
  begin
    Result := Pos(';' + Uppercase(Param) + ';', ';' + Uppercase(Paths) + ';') = 0;
  end;
end;
