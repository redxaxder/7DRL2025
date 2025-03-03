target/wasm32-unknown-unknown/debug/rl2025.wasm: src
	cargo build --target=wasm32-unknown-unknown

debug: target/wasm32-unknown-unknown/debug/rl2025.wasm www
	rm output/*
	cp www/* output
	cp target/wasm32-unknown-unknown/debug/rl2025.wasm output

