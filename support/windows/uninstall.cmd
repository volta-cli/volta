@echo off

SET dirname=%~dp0
cd %dirname:~0,-1%

msiexec /x Notion.msi /l* Uninstall.log
