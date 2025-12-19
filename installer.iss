; Phoenix AGI: Sola Edition Installer Script
; For Inno Setup 6

#define MyAppName "Phoenix AGI: Sola Edition"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "PAGI"
#define MyAppURL "https://pagi.org"
#define MyAppExeName "launcher.cmd"
#define MyAppID "{{E37E71D5-F768-4E37-8F2E-1E3C85FDC0BA}"

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
; Do not use the same AppId value in installers for other applications.
AppId={#MyAppID}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={localappdata}\Phoenix
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputDir=.
OutputBaseFilename=PAGI-SolaSetup
Compression=lzma
SolidCompression=yes
PrivilegesRequired=lowest
SetupIconFile=frontend\public\favicon.ico
UninstallDisplayIcon={app}\frontend\dist\favicon.ico

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "staging\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{commondesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[Code]
function InitializeSetup(): Boolean;
begin
  Result := True;
  
  { Check if the app is already running and close it if needed }
  if CheckForMutexes('Phoenix-Web-Running') then
  begin
    if MsgBox('Sola is currently running. The setup will now close it to proceed with installation. Continue?', 
              mbConfirmation, MB_YESNO) = IDYES then
    begin
      { Try to terminate the running instance }
      TryTerminateProcess('pagi-sola-web.exe');
      Sleep(1000);  { Wait a bit for process to terminate }
    end
    else
      Result := False;  { User chose not to continue }
  end;
end;

procedure TryTerminateProcess(ProcessName: String);
var
  ResultCode: Integer;
begin
  Exec('taskkill.exe', '/F /IM ' + ProcessName, '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
end;
