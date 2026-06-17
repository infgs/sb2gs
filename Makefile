.PHONY: test wasm

TEST_PROJECTS := 1290981008 788908450

test:
	$(foreach project,$(TEST_PROJECTS),cargo run --package sb2gs-cli -- --overwrite --verify --id $(project) tests/$(project).sb3 &&) true

wasm:
	wasm-pack build crates/wasm --target web
