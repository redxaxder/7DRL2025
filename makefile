target/wasm32-unknown-unknown/debug/rl2025.wasm: src
	cargo build --target=wasm32-unknown-unknown


target/wasm32-unknown-unknown/release/rl2025.wasm: src
	cargo build --target=wasm32-unknown-unknown --release

output/debug: target/wasm32-unknown-unknown/debug/rl2025.wasm www
	@rm -rf output
	@mkdir output
	@cp www/* output
	@cp target/wasm32-unknown-unknown/debug/rl2025.wasm output


output/release: target/wasm32-unknown-unknown/release/rl2025.wasm www
	@rm -rf output
	@mkdir output
	@cp www/* output
	@cp target/wasm32-unknown-unknown/release/rl2025.wasm output
	@touch output/release

release: output/release

deploy: release
	butler push output redxaxder/7drl2025:html5
