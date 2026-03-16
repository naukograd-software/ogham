# Schema Compatibility

## Overview

Ogham schemas compile to protobuf and follow protobuf's wire compatibility model. This document defines rules for safe schema evolution, breaking change detection, and cross-version compatibility.

## Schema Evolution Rules

### Safe changes (INFO)

These changes are always safe. Logged but never block.

- Adding a new field to a type
- Adding a new enum value
- Adding a new rpc to a service
- Adding a new type, shape, enum, or annotation
- Adding or changing `<-` mappings (projections are types — mappings are compile-time metadata, not wire)
- Adding or changing annotations

### Unsafe changes requiring `--allow` (WARNING)

These changes don't break wire format but break JSON encoding or generated code.

- Renaming a field (wire uses field numbers, but JSON uses field names)
- Renaming an enum value (wire uses integer, but JSON uses string name)
- Changing a field from optional to non-optional (`T?` → `T`) or vice versa
- Removing or renaming an rpc in a service (gRPC uses method name in HTTP path)

### Breaking changes requiring `--force` (ERROR)

These changes break wire format. Existing clients will fail to deserialize.

- Changing a field number
- Changing a field's wire type (e.g., `int32` → `string`, `message` → `enum`)
- Removing a field without `@reserved` on its field number
- Removing an enum value without `@removed(fallback=...)`
- Changing a field between singular and repeated (`T` ↔ `[]T`)
- Changing or removing the input/output type of an rpc
- Adding or removing `stream` modifier on an rpc

## Breaking Change Detection

The compiler has built-in breaking change detection. It compares the current schema against a previous version and reports violations.

### Usage

```bash
# Compare against a git ref
ogham check breaking --against git:main
ogham check breaking --against git:v1.0.0
ogham check breaking --against git:abc123

# Compare against a local directory
ogham check breaking --against ./previous-schemas/

# Compare against a published version from registry/proxy
ogham check breaking --against github.com/org/schemas@v1.0.0
```

### Flags

| Flag | Description |
|------|-------------|
| (none) | ERROR and WARNING block. INFO logged. |
| `--allow` | Only ERROR blocks. WARNING and INFO logged. |
| `--force` | Nothing blocks. Everything logged. |

### What is compared

The compiler compares **final expanded types** — after shape injection, generic monomorphization, Pick/Omit expansion, and type alias resolution. Shapes, generics, and aliases are not checked directly because they don't exist in the final proto output.

Comparison is done by **fully qualified type name**. A type renamed at the Ogham level but producing the same proto message name is not a breaking change.

### Comparison scope

| Construct | Checked | How |
|-----------|---------|-----|
| Type fields | Yes | Field number, wire type, name, repeated/optional |
| Enum values | Yes | Value number, name, removed status |
| Service rpcs | Yes | Method name, input type, output type, stream modifiers |
| Annotations | No | Annotations are metadata, not wire format |
| `<-` mappings | No | Compile-time metadata, not wire format |
| Shapes | No | Checked indirectly via expanded types |
| Type aliases | No | Checked indirectly via expanded types |
| Generics | No | Checked indirectly via monomorphized types |

## Cross-Version Compatibility

Ogham follows proto3 compatibility semantics. A message encoded with schema v1 can be decoded by schema v2, and vice versa, as long as no ERROR-level breaking changes were made.

### Backward compatibility (new code reads old data)

- New fields not present in old data get their **zero value** (`0`, `""`, `false`, `nil`).
- Removed fields (with `@reserved`) are ignored — their field numbers are never reused.
- New enum values are never encountered in old data.

### Forward compatibility (old code reads new data)

- Unknown fields are **preserved** by default (proto3 behavior). Old code can round-trip data without losing fields it doesn't know about.
- New enum values not recognized by old code are deserialized as `0` (Unspecified). This is why `Unspecified = 0` is implicitly added to every enum.
- New oneof variants not recognized by old code result in an empty oneof (no variant set).
- New rpcs in a service are ignored by old clients — they simply never call them.

### `@removed` enum values

When an enum value is marked `@removed(fallback=X)`:
- The value **stays in the proto enum** (proto enums never delete values).
- Old data containing the removed value is deserialized normally.
- New code receiving the removed value should treat it as the fallback value `X`.
- The compiler enforces that the fallback value is not itself removed (no fallback chains).

### `@reserved` fields

When a field number is reserved with `@reserved(N)`:
- The number `N` is added to the proto `reserved` list.
- No new field can reuse number `N`.
- Old data containing field `N` is silently ignored during deserialization.

### Versioning strategy

Ogham does not prescribe a versioning strategy, but recommends:

1. **Use semver for published schemas.** Major version bump for ERROR-level changes, minor for WARNING-level, patch for INFO-level.
2. **Run `ogham check breaking` in CI.** Compare against the latest published version or the main branch.
3. **Never reuse field numbers.** Use `@reserved` when removing fields.
4. **Never delete enum values.** Use `@removed(fallback=...)` instead.
5. **Add fields with new numbers at the end.** Don't insert into gaps in the numbering.

## Proto Type Mapping Reference

For reference, the wire types that determine compatibility:

| Ogham type | Proto wire type | Compatible with |
|------------|----------------|-----------------|
| `bool` | varint | `int32`, `uint32`, `int64`, `uint64`, `enum` |
| `int32`, `i8`, `int16` | varint | `int64`, `uint32`, `uint64`, `bool`, `enum` |
| `int64`, `int` | varint | `int32`, `uint32`, `uint64`, `bool`, `enum` |
| `uint32`, `uint8`, `uint16`, `byte` | varint | `int32`, `int64`, `uint64`, `bool`, `enum` |
| `uint64`, `uint` | varint | `int32`, `int64`, `uint32`, `bool`, `enum` |
| `float` | 32-bit | — |
| `double` | 64-bit | — |
| `string` | length-delimited | `bytes` |
| `bytes` | length-delimited | `string` |
| `enum` | varint | `int32`, `uint32`, `int64`, `uint64`, `bool` |
| `type` (message) | length-delimited | — |
| `[]T` (repeated) | length-delimited | — (not compatible with singular `T`) |
| `map<K,V>` | length-delimited | — (encoded as `repeated` message with key/value fields) |

Changing between types within the same wire type group is technically wire-compatible but is still flagged as **WARNING** because codegen and JSON behavior differ.
