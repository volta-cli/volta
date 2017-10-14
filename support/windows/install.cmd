@echo off

SET dirname=%~dp0
cd %dirname:~0,-1%

call .\build.cmd

msiexec /i Nodeup.msi /l* Install.log
