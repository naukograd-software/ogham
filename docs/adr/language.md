# Ogham Language Syntax Reference

## Package System

Packages work similarly to Go. Each file starts with a package declaration. Files in the same package can reference each other's types directly.

```
package <name>;
import <path>;
import <path> as <alias>;
```

**Visibility**: names starting with uppercase letters are exported; lowercase names are package-private.

## Namespace Resolution

- **Types** are resolved via `.` — `uuid.UUID`, `std.Timestamp`
- **Annotations** are invoked via `::` — `@database::Table(...)`

## Primitive Types

| Type | Description |
|------|-------------|
| `bool` | Boolean |
| `string` | UTF-8 string |
| `bytes` | Raw bytes |
| `i8` | Signed 8-bit integer |
| `int16` | Signed 16-bit integer |
| `int32` | Signed 32-bit integer |
| `int64` | Signed 64-bit integer |
| `uint8` | Unsigned 8-bit integer |
| `uint16` | Unsigned 16-bit integer |
| `uint32` | Unsigned 32-bit integer |
| `uint64` | Unsigned 64-bit integer |
| `int` | Alias for `int64` |
| `uint` | Alias for `uint64` |
| `byte` | Alias for `uint8` |
| `float` | 32-bit float |
| `double` | 64-bit float |

## Container Types

| Syntax | Description |
|--------|-------------|
| `[]T` | Repeated (array/list) |
| `T?` | Optional |
| `[N]T` | Fixed-size array with N elements |
| `map<K, V>` | Map (K must be comparable) |

## Type

A structure with numbered fields. Supports wire compatibility through explicit field numbers.

```
type Name {
    <type> <field_name> = <field_number>;
}
```

**Type alias** — compile-time synonym:

```
type Name = OtherType;
```

**Generic type** — compile-time monomorphization (not runtime generics):

```
type Name<T> {
    []T data = 1;
}
```

**Nested types** are always references (like protobuf messages), not values. Cyclic dependencies are allowed.

## Shape

A set of fields without numbering. Used as a mixin for composition in a type.

```
shape Name {
    <type> <field_name>;
}
```

**Composition** — a shape can include other shapes:

```
shape Combined {
    ShapeA;
    ShapeB, ShapeC;
}
```

**Injection into a type** — a shape is embedded with an explicit field number range:

```
type Model {
    MyShape(1..4)
    <type> next_field = 5;
}
```

The compiler verifies that the shape fits into the `1..4` range. If the shape grows beyond the range, compilation fails. Keep extra capacity in the range if growth is expected.

## Enum

```
enum Name {
    Value1 = 1;
    Value2 = 2;
}
```

`Unspecified = 0` is added implicitly.

**Removing values**: `@removed(fallback=<non-removed-value>)`. The fallback must reference a non-removed value. Fallback chains are not allowed.

## Oneof

Defined only inside a type. Fields are numbered in the parent type field space.

```
type Model {
    oneof field_name {
        TypeA variant_a = 2;
        TypeB variant_b = 3;
    }
}
```

Multiple fields of the same type with different field numbers are allowed.

## Service & Contract

A service is a group of RPC contracts. A contract defines input and output.

```
service Name {
    contract MethodName(InputType) -> OutputType;
}
```

- `void` means no input or no output
- Inline type `{ fields }` makes the compiler generate `<ContractName>Input` / `<ContractName>Output`
- Generic return types (`Paginated<T>`) use compile-time monomorphization

## Keywords: Pick & Omit

Built-in language keywords (not library features).

**Pick** creates a type from a subset of fields:

```
type Sub = Pick<Original, field1, field2>;
```

**Omit** creates a type excluding fields:

```
type Without = Omit<Original, field1, field2>;
type Without2 = Omit<Original, ShapeName>;
```

For shape-based Omit, matching uses name+type pairs. The compiler emits a warning if nothing is excluded.

## Annotations

### Definition

An annotation is defined in a library with explicit targets and a parameter schema.

```
annotation Name for <target1>|<target2> {
    <type> <param_name>;
    <type>? <param_name> = <default>;
}
```

**Targets**: `shape`, `type`, `field`, `oneof`, `oneof_field`, `enum`, `enum_value`, `service`, `contract`.

A field is optional if it has `?` or a default value.

### Call

```
@<library>::<AnnotationName>(<param>=<value>, ...)
```

### Built-in Annotations

| Annotation | Description | Proto mapping |
|------------|-------------|---------------|
| `@default(<value>)` | Default value. Magic keywords: `now`, `(u)int*.<min,max>` | Custom option `ogham.default` |
| `@cast(<type>)` | Safe type cast | Custom option `ogham.cast` |
| `@removed(fallback=<value>)` | Mark enum value as logically removed | Custom options `ogham.removed` + `ogham.fallback` (the value remains in proto enum, because proto enums do not remove values) |
| `@reserved(<number>)` | Reserve a field number | `reserved <number>;` |

### Proto Target Mapping

| Annotation target | Proto option type |
|-------------------|-------------------|
| `type` | `google.protobuf.MessageOptions` |
| `field` | `google.protobuf.FieldOptions` |
| `oneof` | `google.protobuf.OneofOptions` |
| `oneof_field` | `google.protobuf.FieldOptions` |
| `enum` | `google.protobuf.EnumOptions` |
| `enum_value` | `google.protobuf.EnumValueOptions` |
| `service` | `google.protobuf.ServiceOptions` |
| `contract` | `google.protobuf.MethodOptions` |
| `shape` | `google.protobuf.MessageOptions` (propagates to the message where the shape is injected) |

## Semicolons

A semicolon is required after all declarations: fields, type aliases, enum values, and contracts.

## Protobuf Compatibility

Ogham is fully protobuf-compatible: any `.ogham` schema can be compiled into a valid `.proto` file.

### Type Mapping

| Ogham | Proto |
|-------|-------|
| `i8`, `int16` | `int32` (widening) |
| `int32` | `int32` |
| `int64`, `int` | `int64` |
| `uint8`, `uint16`, `byte` | `uint32` (widening) |
| `uint32` | `uint32` |
| `uint64`, `uint` | `uint64` |
| `bool` | `bool` |
| `string` | `string` |
| `bytes` | `bytes` |
| `float` | `float` |
| `double` | `double` |
| `[]T` | `repeated T` |
| `T?` | `optional T` |
| `[N]T` | `repeated T` (size is a compile-time constraint) |
| `map<K, V>` | `map<K, V>` (keys are always comparable and converted to proto key types) |

### Structural Mapping

| Ogham | Proto |
|-------|-------|
| `type` | `message` |
| `type Alias = T` | Expanded into the target type |
| `type Generic<T>` | Monomorphization into concrete `message` types |
| `enum` | `enum` (all values preserved, `@removed` becomes an option) |
| `shape` | Expanded into `message` fields |
| `Pick<T, ...>` / `Omit<T, ...>` | New `message` with a field subset |
| `oneof` | `oneof` |
| `service` | `service` |
| `contract` | `rpc` |
| `void` | `google.protobuf.Empty` |

### Annotations -> OghamAnnotation

All annotations are serialized via a single `OghamAnnotation` extension backed by `google.protobuf.Struct`:

```protobuf
// ogham/options.proto — part of ogham std
import "google/protobuf/descriptor.proto";
import "google/protobuf/struct.proto";

message OghamAnnotation {
    string name = 1;                     // "database::Table"
    google.protobuf.Struct params = 2;   // { "table_name": "users" }
}

extend google.protobuf.MessageOptions   { repeated OghamAnnotation ogham = 50000; }
extend google.protobuf.FieldOptions     { repeated OghamAnnotation ogham = 50001; }
extend google.protobuf.OneofOptions     { repeated OghamAnnotation ogham = 50002; }
extend google.protobuf.EnumOptions      { repeated OghamAnnotation ogham = 50003; }
extend google.protobuf.EnumValueOptions { repeated OghamAnnotation ogham = 50004; }
extend google.protobuf.ServiceOptions   { repeated OghamAnnotation ogham = 50005; }
extend google.protobuf.MethodOptions    { repeated OghamAnnotation ogham = 50006; }
```

No numbering is needed in annotation declarations: the compiler validates types and `Struct` is used as the transport format. Example:

```
// Ogham source:
@database::Table(table_name="users")
type User { ... }

// Generated .proto:
message User {
    option (ogham) = { name: "database::Table", params: { fields { key: "table_name" value { string_value: "users" } } } };
}
```
