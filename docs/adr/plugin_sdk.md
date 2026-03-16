# Plugin SDK Architecture

## Overview

The Ogham compiler uses two internal representations:

- **AST** (Abstract Syntax Tree) — lossless CST → typed AST, used by the compiler's parser, type checker, linter, and LSP. Defined in Rust, not exposed externally.
- **IR** (Intermediate Representation) — fully resolved, flat representation sent to plugins. Defined as `.proto` files — the source of truth for all SDKs.

AST is internal to the compiler. IR is the public contract between the compiler and plugins.

## Compiler Stack

### Lexer

**logos** — compiles token rules into a single DFA at compile time. Fastest lexer generator in the Rust ecosystem.

### Parser

**Hand-written recursive descent** producing a lossless CST via **rowan** (or **cstree** for `Send + Sync` trees). This is the rust-analyzer / Biome / apollo-rs architecture.

Lossless CST preserves all whitespace, comments, and trivia — essential for LSP features (formatting, refactoring, code actions).

### Typed AST

CST is lowered into a typed Rust AST for semantic analysis. The typed layer can be generated from an **ungrammar** spec.

### LSP

**tower-lsp** + **lsp-types**. LSP works against the CST/AST — not the IR.

### Summary

| Layer | Crate | Role |
|-------|-------|------|
| `logos` | Lexer | Token rules → DFA |
| `rowan` / `cstree` | CST | Lossless concrete syntax tree |
| `ungrammar` | Typed CST | DSL → typed accessor codegen |
| `tower-lsp` | LSP server | Language server framework |
| `lsp-types` | LSP types | Protocol types |

## AST vs IR

| | AST | IR |
|---|---|---|
| **Defined in** | Rust (rowan CST → typed AST) | `.proto` files |
| **Purpose** | Parser output, compiler internals, LSP | Plugin input, code generation |
| **Who uses it** | Compiler, LSP, linters | Plugins via SDK |
| **Content** | Lossless parse tree with unresolved references | Fully resolved, flat, concrete types with full trace |
| **Stability** | Internal, may change between compiler versions | Public contract, semver versioned |
| **Serialization** | None (in-process Rust types) | Protobuf binary over stdin/stdout |

### What the compiler resolves (AST → IR)

| Construct | In AST | In IR |
|-----------|--------|-------|
| Shapes | Shape nodes | Expanded into type fields with assigned numbers + trace back to source shape |
| Generics | Parameterized type nodes | Monomorphized into concrete types |
| Type aliases | Alias nodes | Expanded into target types |
| Pick/Omit | Keyword nodes | Expanded into concrete types |
| Annotation composition | Nested annotation nodes | Recursively expanded to primitives |
| Projection mappings | Unresolved `<-` references | Resolved with validated source paths, full chains unwound |
| Imports | Unresolved paths | Resolved to concrete types across packages |

## Pipeline

```
*.ogham
    ↓ logos (lexer)
    ↓
Token stream
    ↓ hand-written recursive descent
    ↓
Lossless CST (rowan)
    ↓ typed layer (ungrammar)
    ↓
Typed AST (Rust)              ← LSP works here
    ↓ type checker, linter, validator
    ↓ shape expansion, monomorphization, alias resolution
    ↓ Pick/Omit expansion, annotation composition, projection resolution
    ↓
IR (protobuf)
    ↓ serialized as OghamCompileRequest
    ↓ sent to plugin via stdin
    ↓
Plugin (uses SDK to read IR, generate code)
    ↓
OghamCompileResponse (protobuf)
    ↓ sent back via stdout
    ↓
ogham compiler (writes generated files to disk)
```

## IR Design Principles

The IR is designed so plugin authors **never need to resolve references**. Everything is inline and traversable without lookups.

### 1. Recursive types

Field types are not string references — they are full inline type definitions. A plugin can traverse `field → type → fields → type → ...` without looking up anything.

### 2. Inline source mappings

Projection mappings include the full source type and source field inline — not just names. A plugin can access `field.mapping.source_field.annotations` directly. Projection chains are fully unwound.

### 3. Annotation definitions included

Each annotation call includes the full annotation definition (parameter schema, types, defaults, targets). Plugins know what parameters exist without loading annotation packages.

### 4. Back-references

Each type knows which other types reference it. Useful for generating dependency graphs, import lists, or figuring out "who uses this type."

### 5. Resolved enums and services

Enum fields carry the full enum with all values. RPC input/output types are resolved inline — not type names.

### 6. Full trace

Every expanded construct carries a trace back to its origin. Shape fields know which shape they came from. Monomorphized types know their generic source. Pick/Omit results reference the original type. Plugin authors can generate comments, debug info, or documentation linking back to the source.

### What plugin authors get for free

| Task | Approach |
|------|----------|
| Get fields of a message field's type | Traverse inline type — no lookup |
| Get enum values for an enum field | Inline on the field's resolved type |
| Get projection source field's annotations | Inline on the mapping's source field |
| Check if a type is referenced by others | Back-references on the type |
| Get annotation parameter schema | Inline definition on each annotation call |
| Get rpc input type's fields | Inline resolved type on rpc param |
| Traverse nested types | Inline on the parent type |
| Trace a field back to its shape origin | Trace metadata on each expanded field |
| Trace a projection chain to the root source | Full chain on each mapping |

## Proto File Structure

```
proto/ogham/
├── ir/           # IR — public plugin contract (stable, semver)
├── compiler/     # OghamCompileRequest / OghamCompileResponse
└── common/       # Shared types (SourceLocation, etc.)
```

The `.proto` files are the **single source of truth** for the IR. Generated code flows into all SDKs:

| SDK | Generated from |
|-----|---------------|
| Rust (`ogham-proto`, `ogham-plugin-sdk`) | `proto/` via prost/tonic |
| Go (`go/oghamgen`) | `proto/` via protoc-gen-go |
| TypeScript (`ts/oghamgen`) | `proto/` via protoc-gen-ts |

## Plugin SDK

### What the SDK provides

1. **IR types** — generated from `.proto` definitions
2. **Plugin runner** — reads `OghamCompileRequest` from stdin, calls user function, writes `OghamCompileResponse` to stdout
3. **Code generation utilities** — string builders, import managers, file emitters

### Supported languages

| SDK | Package |
|-----|---------|
| Rust | `ogham-plugin-sdk` (crates.io) |
| Go | `github.com/oghamlang/ogham/go/oghamgen` |
| TypeScript | `@ogham/oghamgen` |

### Example (Go)

```go
package main

import "github.com/oghamlang/ogham/go/oghamgen"

func main() {
    oghamgen.Run(func(req *oghamgen.CompileRequest) (*oghamgen.CompileResponse, error) {
        resp := &oghamgen.CompileResponse{}

        for _, typ := range req.Types {
            // fields, annotations, projections — all inline, no lookups
            for _, field := range typ.Fields {
                if field.Mapping != nil {
                    // source type and field are inline, full chain unwound
                    _ = field.Mapping.SourceField.Annotations
                }
                // field type is inline — enum values, nested fields, etc.
                if field.Type.Enum != nil {
                    _ = field.Type.Enum.Values
                }
                // trace back to shape origin if this field came from a shape
                if field.Trace != nil && field.Trace.Shape != nil {
                    _ = field.Trace.Shape.Name
                }
            }

            resp.Files = append(resp.Files, &oghamgen.GeneratedFile{
                Name:    typ.Name + ".go",
                Content: []byte(generatedCode),
            })
        }

        return resp, nil
    })
}
```

## Versioning

| Component | Stability | Policy |
|-----------|-----------|--------|
| `proto/ogham/ir/` | Stable | Semver — breaking changes require major bump |
| `proto/ogham/compiler/` | Stable | Semver — same as IR |
| `proto/ogham/common/` | Stable | Follows IR versioning |
| AST (Rust internals) | Internal | May change between compiler minor versions |

Adding a new IR feature to `.proto` automatically propagates to all SDKs after regeneration. CI validates all languages on every change.
