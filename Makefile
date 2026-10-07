PREFIX ?= $(HOME)/.local
BIN    := target/release/git-identity

build:
	cargo build --release --locked

test:
	cargo test --locked

lint:
	cargo fmt --check
	cargo clippy --all-targets --locked -- -D warnings

install: build
	install -Dm755 $(BIN) $(DESTDIR)$(PREFIX)/bin/git-identity

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/git-identity

clean:
	cargo clean

.PHONY: build test lint install uninstall clean
