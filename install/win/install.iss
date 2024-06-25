[Setup]
AppId={{0449eb4b-4936-42f6-90ab-b58cfc918010}}
AppName=nvm-rs
AppVersion=1.0.0
DefaultDirName={commonpf}\nvm-rs
DefaultGroupName=nvm-rs
OutputDir=../../setup/
OutputBaseFilename=nvm-rs-setup
PrivilegesRequired=admin

; 原来有安装，不强制使用原来的目录
UsePreviousAppDir=no

[Files]
; 包含可执行文件
Source: "..\..\target\release\nvm-rs.exe"; DestDir: "{app}"; DestName: "nvm.exe"; Flags: ignoreversion

; 包含 bin 目录下的所有文件
Source: "..\..\bin\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\..\assets\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
; 开始菜单快捷方式
Name: "{group}\nvm-rs"; Filename: "{app}\nvm-rs.exe"; IconFilename: "{app}\nvm-rs.ico"; Tasks: startmenuicon
; 桌面快捷方式
Name: "{commondesktop}\nvm-rs"; Filename: "{app}\nvm-rs.exe"; IconFilename: "{app}\nvm-rs.ico"; Tasks: desktopicon
; 快速启动栏快捷方式
Name: "{userappdata}\Microsoft\Internet Explorer\Quick Launch\nvm-rs"; Filename: "{app}\nvm-rs.exe"; IconFilename: "{app}\nvm-rs.ico"; Tasks: quicklaunchicon

[Tasks]
Name: "startmenuicon"; Description: "Create a Start Menu icon"; GroupDescription: "Additional icons"; Flags: unchecked
Name: "desktopicon"; Description: "Create a Desktop icon"; GroupDescription: "Additional icons"; Flags: unchecked
Name: "quicklaunchicon"; Description: "Create a Quick Launch icon"; GroupDescription: "Additional icons"; Flags: unchecked

[Run]
Filename: "{app}\nvm-rs.exe"; Description: "Launch nvm-rs"; Flags: nowait postinstall skipifsilent

[Registry]
; 设置环境变量
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "NVM_HOME"; ValueData: "{app}"; Flags: preservestringtype
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "NVM_SYMLINK"; ValueData: "{code:GetNvmSymlink}"; Flags: preservestringtype

[Code]
var
  NvmSymlinkPage: TInputFileWizardPage;
  PathStr: String;

// 判断程序是否已经安装过
function IsAppInstalled: Boolean;
var
  UninstallPath: String;
begin
  // 检查注册表中是否存在卸载信息
  Result := RegQueryStringValue(HKEY_LOCAL_MACHINE, 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{APP_GUID}', 'UninstallString', UninstallPath);
end;
  
procedure InitializeWizard;
var
  ResultCode: Integer;
begin
  LOG('InitializeWizard');

  // 如果已安装，则提示用户是否卸载旧版本
  if IsAppInstalled then
  begin
    if MsgBox('The application is already installed. Do you want to uninstall the existing version?', mbConfirmation, MB_YESNO) = IDYES then
    begin
      // 运行卸载程序
      Exec(ExpandConstant('{uninstallexe}'), '', '', SW_SHOW, ewWaitUntilTerminated, ResultCode);
    end
    else
    begin
      // 取消安装
      WizardForm.Close;
    end;
  end;

  LOG('Select NVM_SYMLINK PATH');
  // 创建文件选择页面，用户选择 NVM_SYMLINK 的位置
  NvmSymlinkPage := CreateInputFilePage(wpSelectDir,
    'Select NVM_SYMLNK Location', 'Where is NVM_SYMLNK located?',
    'Select where NVM_SYMLNK is located, then click Next.');
  NvmSymlinkPage.Add('NVM_SYMLNK:','All files','');

  // 获取当前 PATH 环境变量值
  if not RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'PATH', PathStr) then
    PathStr := '';

  // 如果 PATH 中不存在 %NVM_HOME%，则添加
  if Pos('%NVM_HOME%', PathStr) = 0 then
  begin
    if PathStr <> '' then
      PathStr := PathStr + ';';
    PathStr := PathStr + '%NVM_HOME%';
  end;

  // 如果 PATH 中不存在 %NVM_SYMLINK%，则添加
  if Pos('%NVM_SYMLINK%', PathStr) = 0 then
  begin
    if PathStr <> '' then
      PathStr := PathStr + ';';
    PathStr := PathStr + '%NVM_SYMLINK%';
  end;

  // 更新 PATH 环境变量
  if not RegWriteExpandStringValue(HKEY_CURRENT_USER, 'Environment', 'PATH', PathStr) then
  begin
    MsgBox('Failed to update PATH environment variable!', mbError, MB_OK);
    LOG('Failed to update PATH environment variable!');
  end;
end;

function GetNvmSymlink(Param: string): string;
begin
  Result := NvmSymlinkPage.Values[0];
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  FilePath, Text: string;
begin
  if CurStep = ssPostInstall then
  begin
    FilePath := ExpandConstant('{app}\settings.txt');
    Text := 'root: ' + ExpandConstant('{app}') + #13#10 +
            'symlink: ' + GetNvmSymlink('');
    SaveStringToFile(FilePath, Text, False);
  end;
end;

// 函数用于从 PATH 中移除特定的路径
function RemovePathPart(const OldPath, PartToRemove: String): String;
var
  PathList: TStringList;
  I: Integer;
begin
  Result := '';
  PathList := TStringList.Create;
  try
    PathList.Delimiter := ';';
    PathList.DelimitedText := OldPath;
    
    for I := 0 to PathList.Count - 1 do
    begin
      if CompareText(PathList[I], PartToRemove) <> 0 then
      begin
        if Result <> '' then
          Result := Result + ';';
        Result := Result + PathList[I];
      end;
    end;
  finally
    PathList.Free;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  PathStr: String;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    // 获取当前 PATH 环境变量值
    if not RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'PATH', PathStr) then
      PathStr := '';

    // 从 PATH 中移除 %NVM_HOME%
    PathStr := RemovePathPart(PathStr, '%NVM_HOME%');

    // 从 PATH 中移除 %NVM_SYMLINK%
    PathStr := RemovePathPart(PathStr, '%NVM_SYMLINK%');

    // 更新 PATH 环境变量
    if not RegWriteExpandStringValue(HKEY_CURRENT_USER, 'Environment', 'PATH', PathStr) then
    begin
      MsgBox('Failed to update PATH environment variable!', mbError, MB_OK);
    end;

    // 删除 NVM_HOME 和 NVM_SYMLINK 环境变量
    if not RegDeleteValue(HKEY_CURRENT_USER, 'Environment', 'NVM_HOME') then
    begin
      MsgBox('Failed to delete NVM_HOME environment variable!', mbError, MB_OK);
    end;

    if not RegDeleteValue(HKEY_CURRENT_USER, 'Environment', 'NVM_SYMLINK') then
    begin
      MsgBox('Failed to delete NVM_SYMLINK environment variable!', mbError, MB_OK);
    end;
  end;
end;

[UninstallDelete]
Type: filesandordirs; Name: "{app}"
