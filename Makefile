.PHONY: test
test:
	cargo test -- --test-threads 1

.PHONY: testnc
testnc:
	cargo test -vv -- --test-threads 1 --nocapture

.PHONY: wtest
wtest:
	cargo watch -x "test -- --test-threads 1"
