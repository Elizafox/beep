# Top-level Makefile for beep

PREFIX ?= /usr/local
DESTDIR ?=

.PHONY: all build man install clean

all: build

build:
	cargo build --release

man:
	$(MAKE) -C man

install: build man
	install -Dm755 target/release/beep $(DESTDIR)$(PREFIX)/bin/beep
	install -Dm644 man/beep.1 $(DESTDIR)$(PREFIX)/share/man/man1/beep.1
	install -Dm644 LICENSE $(DESTDIR)$(PREFIX)/share/licenses/beep/LICENSE
	install -Dm644 README.md $(DESTDIR)$(PREFIX)/share/doc/beep/README.md

clean:
	cargo clean
	$(MAKE) -C man clean
