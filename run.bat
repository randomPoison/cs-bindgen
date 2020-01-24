@ECHO OFF

REM Temporary helper script to build the mahjong library as a WASM module and then
REM invoke cs-bindgen-cli on the output. This helps speed up testing during
REM development. We should replace this with a more general (and cross-platform)
REM workflow for testing.

cargo build --target wasm32-unknown-unknown
cargo run -p cs-bindgen-cli -- target\wasm32-unknown-unknown\debug\mahjong.wasm
