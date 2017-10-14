@echo off

SET dirname=%~dp0
cd %dirname:~0,-1%

candle -dNodeupProjectRoot=..\..\ -ext WixUtilExtension Nodeup.wxs
light -ext WixUtilExtension Nodeup.wixobj
