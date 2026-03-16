# Plugin SDK Architecture

## Overview

The Ogham compiler uses two internal representations, both defined as `.proto` files:

- **AST** (Abstract Syntax Tree) — raw parse tree, used by the compiler's parser, lexer, type checker, and linter
- **IR** (Intermediate Representation) — fully resolved, flat representation sent to plugins

Both are protobuf messages. AST is internal to the compiler. IR is the public contract between the compiler and plugins.

## AST vs IR

| | AST | IR |
|---|---|---|
| **Purpose** | Parser output, compiler internals | Plugin input, code generation |
| **Who uses it** | Compiler, LSP, linters | Plugins via SDK |
| **Content** | Raw parse tree with unresolved references | Fully resolved, flat, concrete types |
| **Stability** | Internal, may change between compiler versions | Public contract, semver versioned |

### What the compiler resolves (AST → IR)

| Construct | In AST | In IR |
|-----------|--------|-------|
| Shapes | Shape nodes | Expanded into type fields with assigned numbers |
| Generics | Parameterized type nodes | Monomorphized into concrete types |
| Type aliases | Alias nodes | Expanded into target types |
| Pick/Omit | Keyword nodes | Expanded into concrete types |
| Annotation composition | Nested annotation nodes | Recursively expanded to primitives |
| Projection mappings | Unresolved `<-` references | Resolved with validated source paths |
| Imports | Unresolved paths | Resolved to concrete types across packages |

## Pipeline

```
*.ogham
    ↓ lexer + parser
    ↓
AST (protobuf)
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

Projection mappings include the full source type and source field inline — not just names. A plugin can access `field.mapping.source_field.annotations` directly.

### 3. Annotation definitions included

Each annotation call includes the full annotation definition (parameter schema, types, defaults, targets). Plugins know what parameters exist without loading annotation packages.

### 4. Back-references

Each type knows which other types reference it. Useful for generating dependency graphs, import lists, or figuring out "who uses this type."

### 5. Resolved enums and services

Enum fields carry the full enum with all values. RPC input/output types are resolved inline — not type names.

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

## Proto File Structure

```
ogham/
├── ast/          # AST — compiler internals (unstable)
├── ir/           # IR — public plugin contract (stable, semver)
├── compiler/     # OghamCompileRequest / OghamCompileResponse
└── common/       # Shared types (SourceLocation, etc.)
```

Concrete `.proto` definitions live in the `proto/` directory of the Ogham repository and are the source of truth for SDK generation.

## Plugin SDK

Each language SDK is generated from `ogham/ir/*.proto` and `ogham/compiler/*.proto`, plus hand-written utilities.

### What the SDK provides

1. **IR types** — generated from proto definitions
2. **Plugin runner** — reads `OghamCompileRequest` from stdin, calls user function, writes `OghamCompileResponse` to stdout
3. **Code generation utilities** — string builders, import managers, file emitters

### Supported languages

| SDK | Package |
|-----|---------|
| Go | `github.com/ogham/plugin-sdk-go` |
| TypeScript | `@ogham/plugin-sdk` |
| Rust | `ogham-plugin-sdk` |

### Example (Go)

```go
package main

import "github.com/ogham/plugin-sdk-go/ogham"

func main() {
    ogham.Run(func(req *ogham.CompileRequest) (*ogham.CompileResponse, error) {
        resp := &ogham.CompileResponse{}

        for _, typ := range req.Types {
            // fields, annotations, projections — all inline, no lookups
            for _, field := range typ.Fields {
                if field.Mapping != nil {
                    // source type and field are inline
                    _ = field.Mapping.SourceField.Annotations
                }
                // field type is inline — enum values, nested fields, etc.
                if field.Type.Enum != nil {
                    _ = field.Type.Enum.Values
                }
            }

            resp.Files = append(resp.Files, &ogham.GeneratedFile{
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
| `ogham/ir/` | Stable | Semver — breaking changes require major bump |
| `ogham/compiler/` | Stable | Semver — same as IR |
| `ogham/ast/` | Internal | May change between compiler minor versions |
| `ogham/common/` | Stable | Follows IR versioning |

## Proto Files as Source of Truth

The `.proto` files are the **single source of truth** for both AST and IR. They are:

- Versioned alongside the compiler
- Published to the Ogham registry
- Used to generate SDKs via standard `protoc` tooling

Adding a new IR feature automatically propagates to all SDKs after regeneration.
