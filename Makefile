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

# make release V=1.1.0
# Bumps the version, dates the "Unreleased" changelog section, commits and
# tags. Pushing the tag to Forgejo (and the mirror) triggers the releases.
release:
	@test -n "$(V)" || { echo "usage: make release V=x.y.z"; exit 1; }
	@git diff --quiet && git diff --cached --quiet || { echo "commit your changes first"; exit 1; }
	@grep -q '^## Unreleased' CHANGELOG.md || { echo "CHANGELOG.md has no '## Unreleased' section"; exit 1; }
	sed -i.bak 's/^version = ".*"/version = "$(V)"/' Cargo.toml && rm Cargo.toml.bak
	sed -i.bak "s/^## Unreleased/## $(V) ($$(date +%F))/" CHANGELOG.md && rm CHANGELOG.md.bak
	cargo update --workspace --offline
	git commit -m "chore: release v$(V)" Cargo.toml Cargo.lock CHANGELOG.md
	git tag -a "v$(V)" -m "v$(V)"
	@echo "Now run: git push --follow-tags"

.PHONY: build test lint install uninstall clean release
