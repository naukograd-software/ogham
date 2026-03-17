# Repository Structure

`oghamlang/ogham` — the Ogham language: compiler, LSP, proto definitions, and plugin SDKs for all supported languages. Everything lives in one repository so CI validates the full stack on every change.

## Layout

```
ogham/
├── crates/                  # Rust workspace
│   ├── ogham-cli/           # CLI binary (`ogham`)
│   ├── ogham-compiler/      # Lexer (logos), parser (rowan), type checker, linter, AST → IR lowering
│   ├── ogham-core/          # Shared types and utilities
│   ├── ogham-lsp/           # Language Server Protocol implementation (tower-lsp)
│   ├── oghamgen/            # Rust Plugin SDK (oghamgen crate)
│   ├── ogham-gen-proto/     # Plugin: export .proto files from .ogham schemas
│   └── ogham-proto/         # Generated Rust code from proto/ (prost/tonic)
│
├── proto/                   # Protobuf definitions — source of truth for IR
│   ├── ogham/               # .proto files (ir/, compiler/, common/)
│   ├── assets/              # easyp templates (Cargo.toml.tmpl, etc.)
│   └── easyp.yaml           # easyp generation config
│
├── std/                     # Standard library — Ogham source files
│   ├── uuid/                # github.com/oghamlang/std/uuid — UUID
│   ├── ulid/                # github.com/oghamlang/std/ulid — ULID
│   ├── time/                # github.com/oghamlang/std/time — Timestamp, ProtoTimestamp, Date, TimeOfDay, DateTime, TimeZone
│   ├── duration/            # github.com/oghamlang/std/duration — Duration, ProtoDuration
│   ├── decimal/             # github.com/oghamlang/std/decimal — Decimal
│   ├── geo/                 # github.com/oghamlang/std/geo — LatLng, BoundingBox, GeoPoint
│   ├── empty/               # github.com/oghamlang/std/empty — Empty (use `void` in RPCs instead)
│   ├── fieldmask/           # github.com/oghamlang/std/fieldmask — FieldMask (partial updates)
│   ├── money/               # github.com/oghamlang/std/money — Money (amount + ISO 4217 currency)
│   ├── rpc/                 # github.com/oghamlang/std/rpc — CursorPagination, PageRequest, Sortable, RequestContext, Status, ResponseMeta
│   ├── any/                 # github.com/oghamlang/std/any — Any (type_url + serialized bytes)
│   ├── struct/              # github.com/oghamlang/std/struct — Struct, Value, ListValue (dynamic JSON-like data)
│   ├── wrappers/            # github.com/oghamlang/std/wrappers — BoolValue, StringValue, Int64Value, ...
│   └── validate/            # github.com/oghamlang/std/validate — Required, Length, Pattern, Range, Items, NotEmpty
│
├── go/
│   └── oghamgen/            # Go Plugin SDK (github.com/oghamlang/ogham/go/oghamgen)
│
├── ts/
│   └── oghamgen/            # TypeScript Plugin SDK (@ogham/oghamgen)
│
├── docs/
│   ├── adr/                 # Architecture Decision Records
│   │   ├── syntax/          # Ogham syntax examples and EBNF grammar
│   │   ├── language.md      # Language specification
│   │   ├── package.md       # Package management & module system
│   │   ├── plugin_sdk.md    # Plugin SDK architecture (AST vs IR, pipeline)
│   │   ├── cmd.md           # CLI commands reference
│   │   ├── compatibility.md # Breaking change detection
│   │   ├── validation.md    # Annotation-based validation
│   │   └── wire.md          # Wire formats and serialization
│   └── repository.md        # ← this file
│
├── examples/
│   └── store/               # Example: online store schemas (5 files, 3 services, 12 RPCs)
│
├── Cargo.toml               # Workspace manifest
├── Cargo.lock
├── Makefile                  # Build: make build (→ bin/), make test, make ci
├── AGENTS.md                # Agent workflow and code style guidelines
├── LICENSE
└── .gitignore
```

## Components

### Compiler (`crates/ogham-compiler`)

Logos lexer, hand-written recursive-descent parser producing a lossless CST (rowan), typed AST layer, type checker, linter, and AST → IR lowering. AST is pure Rust — not defined in proto. See [adr/plugin_sdk.md](adr/plugin_sdk.md) for the full pipeline.

### CLI (`crates/ogham-cli`)

The `ogham` binary. Schema validation (`check`), code generation (`generate`), package management (`get`, `install`, `update`, `vendor`), breaking change detection (`breaking`), IR debug dump (`dump`). See [adr/cmd.md](adr/cmd.md).

### LSP (`crates/ogham-lsp`)

Language server for editor integration: diagnostics, hover, completion, go-to-definition. Works against the CST/AST (not IR).

### Core (`crates/ogham-core`)

Shared types and utilities used across compiler, CLI, and LSP.

### Proto definitions (`proto/`)

The `.proto` files are the single source of truth for IR and compiler protocol messages. No AST definitions — AST lives only in Rust. Generated code flows into `crates/ogham-proto` (Rust), `go/oghamgen/` (Go), and `ts/oghamgen/` (TypeScript).

Regenerate after changing `.proto` files:

```bash
make proto
```

### Rust Plugin SDK (`crates/oghamgen`)

Part of the Cargo workspace. Depends on `ogham-proto` for IR types, adds the plugin runner (`run()`) and code generation utilities (`CodeWriter`, case converters). Published as `oghamgen` on crates.io.

### Proto Export Plugin (`crates/ogham-gen-proto`)

Built-in plugin that generates `.proto3` files from Ogham schemas. Uses the `oghamgen` SDK — serves as a reference implementation for plugin authors. Run via `ogham generate --plugin=proto`.

### Registry Proxy (planned: `ogham-proxy`)

Separate binary — serves packages over HTTP (GOPROXY-compatible protocol). Not part of the `ogham` CLI. See [package.md](adr/package.md) for proxy protocol details. The CLI resolves dependencies via git clone or local path when no proxy is configured.

### Go Plugin SDK (`go/oghamgen`)

Go module with its own `go.mod`. Import path: `github.com/oghamlang/ogham/go/oghamgen`. IR types generated from `proto/` + hand-written plugin runner / codegen helpers.

### TypeScript Plugin SDK (`ts/oghamgen`)

npm package with its own `package.json`. Published as `@ogham/oghamgen`. IR types generated from `proto/` + hand-written plugin runner / codegen helpers.

### SDK summary

| Directory | Published as | Language |
|-----------|-------------|----------|
| `crates/oghamgen` | `oghamgen` | Rust |
| `go/oghamgen` | `github.com/oghamlang/ogham/go/oghamgen` | Go |
| `ts/oghamgen` | `@ogham/oghamgen` | TypeScript |

All SDKs are tested in CI alongside the compiler — a proto change that breaks an SDK is caught immediately.

## Build

`Makefile` is the single entry point for all build, lint, test, and code generation commands. Run `make help` to see available targets.

## Why monorepo

- **Proto changes are validated end-to-end.** Changing an IR message in proto regenerates all SDKs in one PR. CI catches breakage across languages before merge.
- **Compiler and SDK versions stay in sync.** No cross-repo version matrix. One release tag covers the compiler and all SDKs.
- **Single CI pipeline.** Lint, test, and publish everything from one place.
