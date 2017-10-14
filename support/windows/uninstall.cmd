@echo off

SET dirname=%~dp0
cd %dirname:~0,-1%

msiexec /x Nodeup.msi /l* Uninstall.log
