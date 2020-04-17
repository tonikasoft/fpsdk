:: Usage:
:: install.win.bat name destination_name [plugins_dir]

@echo off

set name=%1
set dest_name=%2
set plugins_dir="C:\Program Files\Image-Line\FL Studio 20\Plugins\Fruity"
set args_count=0
set flag=false

for %%x in (%*) do Set /A args_count+=1

if %args_count% lss 2 set flag=true
if %args_count% gtr 3 set flag=true
if "%flag%"=="true" (
    echo "Usage: install.win.bat name destination_name [plugins_dir]"
    exit /B 87
)

if %args_count%==3 (
    set plugins_dir=%3
)

rd /s /q %plugins_dir%\Effects\%dest_name%
md %plugins_dir%\Effects\%dest_name%
move target\release\examples\%name%.dll %plugins_dir%\Effects\%dest_name%\%dest_name%_x64.dll
