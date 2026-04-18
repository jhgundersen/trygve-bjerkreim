PREFIX  ?= $(HOME)/.local
BINARY   = tbv
CARGO    = tbv-rs
EXAMPLES = examples

.PHONY: build release install uninstall test clean

build:
	cd $(CARGO) && cargo build

release:
	cd $(CARGO) && cargo build --release

install: release
	@mkdir -p $(PREFIX)/bin
	@install -m 755 $(CARGO)/target/release/$(BINARY) $(PREFIX)/bin/$(BINARY)
	@echo "Installed $(PREFIX)/bin/$(BINARY)"

uninstall:
	@rm -f $(PREFIX)/bin/$(BINARY)
	@echo "Removed $(PREFIX)/bin/$(BINARY)"

test: build
	@for f in $(EXAMPLES)/hei_verd.tb $(EXAMPLES)/fizzbuzz.tb \
	           $(EXAMPLES)/fibonacci.tb $(EXAMPLES)/lister.tb \
	           $(EXAMPLES)/funksjonar.tb; do \
	  echo "--- $$f ---"; \
	  $(CARGO)/target/debug/$(BINARY) $$f || exit 1; \
	done
	@echo "Alle prøvar gjekk gjennom."

clean:
	cd $(CARGO) && cargo clean
