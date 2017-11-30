@echo off

SET dirname=%~dp0
cd %dirname:~0,-1%

call .\build.cmd

msiexec /i Standup.msi /l* Install.log
