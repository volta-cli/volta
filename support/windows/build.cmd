@echo off

SET dirname=%~dp0
cd %dirname:~0,-1%

candle -dStandupProjectRoot=..\..\ -ext WixUtilExtension Standup.wxs
light -ext WixUtilExtension Standup.wixobj
