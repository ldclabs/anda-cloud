BUILD_ENV := rust

.PHONY: build-wasm build-did

lint:
	@cargo fmt
	@cargo clippy --all-targets --all-features

fix:
	@cargo clippy --fix --workspace --tests

test:
	@AWS_LC_SYS_NO_ASM=1 cargo test --workspace --all-features -- --nocapture

# cargo install ic-wasm
build-wasm:
	@cargo build --release --target wasm32-unknown-unknown --package anda_registry_canister

# cargo install candid-extractor
build-did:
	candid-extractor target/wasm32-unknown-unknown/release/anda_registry_canister.wasm > rs/anda_registry_canister/anda_registry_canister.did
	dfx generate
