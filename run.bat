@ECHO OFF

REM Temporary helper script to generate the dylib, generate the C# bindings, and
REM copy both over to the Unity test project. This helps speed up testing during
REM development. We should replace this with a more general (and cross-platform)
REM workflow for testing.

cargo build --target wasm32-unknown-unknown
cargo run -p cs-bindgen-cli -- -o ../DotNetGamePrototype/DotNetGameClient/Packages/com.synapse-games.mahjong/Mahjong.cs target/wasm32-unknown-unknown/debug/mahjong.wasm

cargo build
xcopy /y target\debug\mahjong.dll ..\DotNetGamePrototype\DotNetGameClient\Packages\com.synapse-games.mahjong
