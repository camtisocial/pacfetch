PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
BINARY = pacfetch

.PHONY: build install uninstall clean

build:
	cargo build --release

install: build
	install -Dm755 target/release/$(BINARY) $(DESTDIR)$(BINDIR)/$(BINARY)

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/$(BINARY)

clean:
	cargo clean
