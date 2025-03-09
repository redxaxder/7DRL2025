target/wasm32-unknown-unknown/debug/rl2025.wasm: src
	cargo build --target=wasm32-unknown-unknown


target/wasm32-unknown-unknown/release/rl2025.wasm: src
	cargo build --target=wasm32-unknown-unknown --release

output/debug: target/wasm32-unknown-unknown/debug/rl2025.wasm www
	@rm -rf output
	@mkdir output
	@cp www/* output
	@cp -r bgm output
	@cp target/wasm32-unknown-unknown/debug/rl2025.wasm output


output/release: target/wasm32-unknown-unknown/release/rl2025.wasm www
	@rm -rf output
	@mkdir output
	@cp www/* output
	@cp -r bgm output
	@cp target/wasm32-unknown-unknown/release/rl2025.wasm output
	@touch output/release

debug: output/debug

release: output/release

deploy: release
	butler push output redxaxder/tilers-adventure:html5
