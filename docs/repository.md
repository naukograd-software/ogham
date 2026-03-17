# Repository Structure

`oghamlang/ogham` — the Ogham language: compiler, LSP, proto definitions, and plugin SDKs for all supported languages. Everything lives in one repository so CI validates the full stack on every change.

## Layout

```
ogham/
├── crates/                  # Rust workspace
│   ├── ogham-cli/           # CLI binary (`ogham`)
│   ├── ogham-compiler/      # Lexer, parser, type checker, semantic analysis, IR lowering, package manager
│   ├── ogham-core/          # Shared types and utilities
│   ├── ogham-lsp/           # Language Server Protocol implementation (tower-lsp)
│   ├── oghamgen/            # Rust Plugin SDK (oghamgen crate)
│   ├── ogham-gen-proto/     # Plugin: export .proto files from .ogham schemas
│   └── ogham-proto/         # Generated Rust code from proto/ (prost/tonic)
│
├── proto/                   # Protobuf definitions — source of truth for IR
│   ├── oghamproto/          # .proto files (ir/, compiler/, common/)
│   ├── assets/              # easyp templates (Cargo.toml.tmpl, etc.)
│   └── easyp.yaml           # easyp generation config (Rust + Go + TS)
│
├── go/                      # Go module (github.com/oghamlang/go)
│   ├── oghamproto/          # Generated Go proto types (protoc-gen-go + protoc-gen-go-grpc)
│   ├── oghamgen/            # Go Plugin SDK — Run(), CodeWriter, name converters
│   ├── go.mod
│   └── go.sum
│
├── ts/                      # TypeScript / Node.js package (@ogham/sdk)
│   ├── oghamproto/          # Generated TS proto types (@bufbuild/protobuf)
│   ├── oghamgen/            # TS Plugin SDK — run(), CodeWriter, name converters
│   ├── package.json
│   └── tsconfig.json
│
├── std/                     # Standard library — Ogham source files
│   ├── uuid/                # github.com/oghamlang/std/uuid — UUID, UUIDString
│   ├── ulid/                # github.com/oghamlang/std/ulid — ULID, ULIDString
│   ├── time/                # github.com/oghamlang/std/time — Timestamp, ProtoTimestamp, Date, TimeOfDay, DateTime, TimeZone
│   ├── duration/            # github.com/oghamlang/std/duration — Duration, ProtoDuration
│   ├── decimal/             # github.com/oghamlang/std/decimal — Decimal
│   ├── geo/                 # github.com/oghamlang/std/geo — LatLng, BoundingBox, GeoPoint
│   ├── empty/               # github.com/oghamlang/std/empty — Empty
│   ├── fieldmask/           # github.com/oghamlang/std/fieldmask — FieldMask
│   ├── money/               # github.com/oghamlang/std/money — Money
│   ├── rpc/                 # github.com/oghamlang/std/rpc — CursorPagination, PageRequest, Sortable, RequestContext, Status, ResponseMeta
│   ├── any/                 # github.com/oghamlang/std/any — Any
│   ├── struct/              # github.com/oghamlang/std/struct — Struct, Value, ListValue
│   ├── wrappers/            # github.com/oghamlang/std/wrappers — BoolValue, StringValue, ...
│   └── validate/            # github.com/oghamlang/std/validate — Required, Length, Pattern, Range, Items, NotEmpty
│
├── examples/
│   └── store/               # Example: online store schemas (5 files, 3 services, 12 RPCs)
│
├── docs/
│   ├── adr/                 # Architecture Decision Records
│   └── repository.md        # ← this file
│
├── Cargo.toml               # Rust workspace manifest
├── Makefile                  # make test (Rust+Go+TS), make build, make ci, make proto
├── LICENSE
└── .gitignore
```

## Components

### Compiler (`crates/ogham-compiler`)

Logos lexer, hand-written recursive-descent parser producing a lossless CST (rowan), typed AST layer, 12 semantic analysis passes, IR inflation to proto, package manager (MVS, transitive deps, git/path sources), and breaking change detection. AST is pure Rust — not in proto. See [adr/plugin_sdk.md](adr/plugin_sdk.md) for the full pipeline.

### CLI (`crates/ogham-cli`)

The `ogham` binary:
- `ogham generate` — compile schemas + run plugins (reads `ogham.gen.yaml`)
- `ogham check` — compile + validate without running plugins
- `ogham breaking` — detect breaking changes against a git ref or directory
- `ogham dump` — dump compiled IR as JSON for debugging
- `ogham get/install/update/vendor` — package management

See [adr/cmd.md](adr/cmd.md).

### LSP (`crates/ogham-lsp`)

Full-featured language server (tower-lsp): diagnostics (parse + semantic), hover, go-to-definition (cross-file + std), find all references, completion (context-aware + std types), document symbols, workspace symbols, rename, formatting, semantic highlighting, inlay hints, signature help, code actions.

### Proto definitions (`proto/`)

The `.proto` files in `proto/oghamproto/` are the single source of truth for IR and compiler protocol. easyp generates code for all three languages:
- Rust → `crates/ogham-proto/` (prost/tonic)
- Go → `go/oghamproto/` (protoc-gen-go)
- TypeScript → `ts/oghamproto/` (protoc-gen-es)

Regenerate after changing `.proto` files:

```bash
make proto
```

### Rust Plugin SDK (`crates/oghamgen`)

`run()` stdin/stdout plugin runner, `CodeWriter` with indentation, name case converters. Published as `oghamgen` on crates.io.

### Go Plugin SDK (`go/oghamgen`)

`Run()` stdin/stdout plugin runner, `CodeWriter`, `ToPascalCase`/`ToSnakeCase`. Import: `github.com/oghamlang/go/oghamgen`.

### TypeScript Plugin SDK (`ts/oghamgen`)

`run()` stdin/stdout plugin runner (Node.js), `CodeWriter` class, name converters. Published as `@ogham/sdk`.

### Proto Export Plugin (`crates/ogham-gen-proto`)

Built-in plugin that generates `.proto3` files from Ogham schemas. Reference implementation for plugin authors. Run via `ogham generate --plugin=proto`.

### Registry Proxy (planned: `ogham-proxy`)

Separate binary — serves packages over HTTP (GOPROXY-compatible). See [package.md](adr/package.md).

### SDK summary

| Directory | Published as | Language |
|-----------|-------------|----------|
| `crates/oghamgen` | `oghamgen` | Rust |
| `go/oghamgen` | `github.com/oghamlang/go/oghamgen` | Go |
| `ts/oghamgen` | `@ogham/sdk` | TypeScript |

## Build

```bash
make help          # show all targets
make proto         # regenerate proto (Rust + Go + TS)
make test          # run all tests (Rust + Go + TS)
make test-rust     # Rust only
make test-go       # Go only
make test-ts       # TypeScript only
make build         # release build → bin/
make install       # build + copy to ~/.ogham/bin/
make ci            # fmt + clippy + all tests
```

## Why monorepo

- **Proto changes are validated end-to-end.** Changing a `.proto` file regenerates Rust, Go, and TS types. CI catches breakage across all languages before merge.
- **Compiler and SDK versions stay in sync.** One release tag covers everything.
- **Single CI pipeline.** `make ci` runs formatting, lints, and all tests in one command.
