.PHONY: run
run:
	cargo run --release

.PHONY: test
test:
	cargo test --bin emergent -- --test-threads 1 --nocapture

.PHONY: wtest
wtest:
	cargo watch -x "test --bin emergent -- --test-threads 1"

.PHONY: watch
watch:
	cargo watch -x "build --all-targets"

