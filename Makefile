default: help

.PHONY: help
help: # Show available Make targets
	@grep -E '^[a-zA-Z0-9_.-]+:.*#' Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[0m:$$(echo $$l | cut -f 2- -d'#')\n"; done

.PHONY: proto
proto: # Generate protobuf and Rust artifacts with easyp
	cd proto && easyp generate

.PHONY: protocols
protocols: proto # Alias for proto generation

.PHONY: check
check: # Run cargo check for all workspace crates
	cargo check --workspace

.PHONY: fmt
fmt: # Format all workspace crates with rustfmt
	cargo fmt --all

.PHONY: clippy
clippy: # Run clippy for all workspace crates
	cargo clippy --workspace --all-targets --all-features

.PHONY: test
test: # Run all workspace tests
	cargo test --workspace

.PHONY: ci
ci: fmt clippy test # Run formatting, lints, and tests
