PREFIX ?= ~/.local

install:
	install -Dm755 git-identity.sh $(PREFIX)/bin/git-identity

uninstall:
	rm -f $(PREFIX)/bin/git-identity

.PHONY: install uninstall
