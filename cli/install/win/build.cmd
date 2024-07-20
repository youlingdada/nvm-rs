@echo off
setlocal

rem 检查 Inno Setup Compiler 是否安装
where ISCC >nul 2>nul
if %ERRORLEVEL%==0 (
    ISCC install.iss
) else (
    echo Inno Setup Compiler is not installed.
)

endlocal
