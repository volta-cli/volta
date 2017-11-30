@echo off

SET dirname=%~dp0
cd %dirname:~0,-1%

msiexec /x Standup.msi /l* Uninstall.log
