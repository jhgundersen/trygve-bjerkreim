PREFIX  ?= $(HOME)/.local
BINARY   = tbv
CARGO    = tbv-rs
EXAMPLES = examples

TARGETS = x86_64-unknown-linux-musl \
          aarch64-unknown-linux-musl \
          x86_64-apple-darwin \
          aarch64-apple-darwin

.PHONY: build release install uninstall test dist clean tag

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

# Build binaries for all platforms using `cross` (requires Docker).
# Install cross with: cargo install cross
dist: dist/
	@command -v cross >/dev/null 2>&1 || (echo "cross ikkje installert. Køyr: cargo install cross" && exit 1)
	@for t in $(TARGETS); do \
	  echo "Byggjer $$t …"; \
	  cross build --release --manifest-path $(CARGO)/Cargo.toml --target $$t; \
	  cp $(CARGO)/target/$$t/release/$(BINARY) dist/$(BINARY)-$$t; \
	  echo "  → dist/$(BINARY)-$$t"; \
	done

dist/:
	@mkdir -p dist

# Tag a release and push to trigger the GitHub Actions release workflow.
# Usage: make tag VERSION=v0.2.0
tag:
	@test -n "$(VERSION)" || (echo "Usage: make tag VERSION=v1.0.0" && exit 1)
	git tag $(VERSION)
	git push origin $(VERSION)
	@echo "Tag $(VERSION) pushed — GitHub Actions bygger no release."

clean:
	cd $(CARGO) && cargo clean
	rm -rf dist/
