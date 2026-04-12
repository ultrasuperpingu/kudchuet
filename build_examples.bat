cargo build --target wasm32-unknown-unknown -p chess@0.1 --release
wasm-bindgen target/wasm32-unknown-unknown/release/chess.wasm --out-dir docs/chess --target web

cargo build --target wasm32-unknown-unknown -p chinese_checkers --release
wasm-bindgen target/wasm32-unknown-unknown/release/chinese_checkers.wasm --out-dir docs/chinese_checkers --target web

cargo build --target wasm32-unknown-unknown -p three_musketeers --release
wasm-bindgen target/wasm32-unknown-unknown/release/three_musketeers.wasm --out-dir docs/three_musketeers --target web


pause