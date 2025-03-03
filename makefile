debug: target/wasm32-unknown-unknown/debug/rl2025.wasm www
	cargo build --target=wasm32-unknown-unknown
	rm output/*
	cp www/* output
	cp target/wasm32-unknown-unknown/debug/rl2025.wasm output

