default: help

.PHONY: help
help: # Show available Make targets
	@grep -E '^[a-zA-Z0-9_.-]+:.*#' Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[0m:$$(echo $$l | cut -f 2- -d'#')\n"; done

.PHONY: proto
proto: # Generate protobuf artifacts (Rust + Go + TS) with easyp
	cd proto && easyp generate

.PHONY: check
check: # Run cargo check for all workspace crates
	cargo check --workspace

.PHONY: fmt
fmt: # Format Rust code with rustfmt
	cargo fmt --all

.PHONY: clippy
clippy: # Run clippy for all workspace crates
	cargo clippy --workspace --all-targets --all-features

.PHONY: test
test: test-rust test-go test-ts # Run all tests (Rust + Go + TS)

.PHONY: test-rust
test-rust: # Run Rust tests
	cargo test --workspace

.PHONY: test-go
test-go: # Run Go tests
	cd go && go test ./...

.PHONY: test-ts
test-ts: # Run TypeScript tests
	cd ts && npm test

.PHONY: build
build: # Build all binaries into ./bin/
	cargo build --release
	cd go && go build -o ../bin/ogham-gen-go ./ogham-gen-go/
	@mkdir -p bin
	@cp target/release/ogham bin/ 2>/dev/null || true
	@cp target/release/ogham-lsp bin/ 2>/dev/null || true
	@cp target/release/ogham-gen-proto bin/ 2>/dev/null || true
	@echo "Binaries:"
	@ls -lh bin/

.PHONY: install
install: build # Build and copy binaries to $$OGHAM_BIN or ~/.ogham/bin
	@mkdir -p $${OGHAM_BIN:-$$HOME/.ogham/bin}
	@cp bin/* $${OGHAM_BIN:-$$HOME/.ogham/bin}/
	@echo "Installed to $${OGHAM_BIN:-$$HOME/.ogham/bin}"

.PHONY: example
example: build # Generate Proto code for examples/golden, then run protoc-gen-go
	./bin/ogham generate --dir examples/golden
	@if command -v protoc > /dev/null 2>&1; then \
		mkdir -p examples/golden/gen/pbgo; \
		protoc \
			--proto_path=examples/golden/gen/proto \
			--go_out=examples/golden/gen/pbgo \
			--go_opt=paths=source_relative \
			examples/golden/gen/proto/*.proto; \
	else \
		echo "protoc not found — skipping Go proto generation"; \
	fi

.PHONY: ci
ci: fmt clippy test # Run formatting, lints, and all tests (Rust + Go + TS)
