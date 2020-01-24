@ECHO OFF

REM Temporary helper script for copying the generated dylib over to the Unity
REM project for testing. This script is specific to the mahjong test library and
REM should be removed once that library is split off into its own project. We'll
REM also want to replace this script with a better workflow for building a Rust
REM library into a dylib and then importing the dylib into Unity.

cargo build
xcopy /y target\debug\mahjong.dll ..\dotNetGame\DotNetGameClient\Packages\com.synapse-games.mahjong
