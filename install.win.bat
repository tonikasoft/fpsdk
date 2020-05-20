:: Usage:
:: install.win.bat name destination_name type_of_plugin [plugins_dir]

@echo off

set name=%1
set dest_name=%2
set plugins_dir="C:\Program Files\Image-Line\FL Studio 20\Plugins\Fruity"
set args_count=0
set incorrect_args_count=false
set type_flag=false

for %%x in (%*) do Set /A args_count+=1

if %args_count% lss 3 set incorrect_args_count=true
if %args_count% gtr 4 set incorrect_args_count=true
if "%incorrect_args_count%"=="true" (
    echo "Usage: install.win.bat name destination_name type_of_plugin [plugins_dir]"
    exit /B 87
)

if "%3"=="-e" (
    set type=Effects
    set type_flag=true
) 

if "%3"=="-g" (
    set type=Generators
    set type_flag=true
)

if "%type_flag%"=="false" (
    echo "please type '-e' or '-g'"
    exit /B 87
)

if %args_count%==4 (
    set plugins_dir=%4
)

rd /s /q %plugins_dir%\%type%\%dest_name%
md %plugins_dir%\%type%\%dest_name%
move target\release\examples\%name%.dll %plugins_dir%\%type%\%dest_name%\%dest_name%_x64.dll
