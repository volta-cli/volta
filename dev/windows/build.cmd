@echo off

SET dirname=%~dp0
cd %dirname:~0,-1%

candle -dNotionProjectRoot=..\..\ -ext WixUtilExtension Notion.wxs
light -ext WixUtilExtension Notion.wixobj
