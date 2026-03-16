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
| `map<K, V>` | Map (K must be a proto-compatible key type: `bool`, `string`, `int32`, `int64`, `uint32`, `uint64` or their aliases) |

## Type

A structure with numbered fields. Supports wire compatibility through explicit field numbers.

```
type Name {
    <type> <field_name> = <field_number>;
}
```

**Type alias** — compile-time synonym. Annotations on the alias apply wherever the alias is used and are preserved during proto expansion:

```
type Name = OtherType;
```

**Generic type** — compile-time monomorphization (not runtime generics). Each instantiation produces a concrete proto message named by concatenating the generic name with the type arguments: `Paginated<User>` → `PaginatedUser`, `Pair<User, Order>` → `PairUserOrder`.

```
type Name<T> {
    []T data = 1;
}
```

**Nested definitions** — types, enums, and shapes can be defined inside a type (like protobuf nested messages):

```
type Order {
    int64 id = 1;
    Status status = 2;
    Address shipping = 3;

    enum Status {
        Pending = 1;
        Shipped = 2;
        Delivered = 3;
    }

    type Address {
        string street = 1;
        string city = 2;
        string zip = 3;
    }
}
```

Nested definitions are scoped to the parent type: `Order.Status`, `Order.Address`. They can be referenced from outside by qualified name (`Order.Address`). Visibility rules apply — uppercase names are exported, lowercase are private to the package.

**Nested types** are always references (like protobuf messages), not values. Cyclic dependencies are allowed.

## Shape

A set of fields without numbering. Used as a mixin for composition in a type.

```
shape Name {
    <type> <field_name>;
}
```

**Generic shape** — compile-time monomorphization, same as generic types:

```
shape Wrapper<T> {
    T value;
}
```

**Composition** — a shape can include other shapes:

```
shape Combined {
    ShapeA;
    ShapeB, ShapeC;
}
```

Fields are expanded in declaration order: `shape Combined { ShapeA; ShapeB; }` expands ShapeA fields first, then ShapeB. If two shapes have a field with the same name, it is a compile error — including cases where both shapes include a common third shape (diamond includes).

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

## Service & RPC

A service is a group of RPCs. An RPC defines input and output.

```
service Name {
    rpc MethodName(InputType) -> OutputType;
}
```

- `void` means no input or no output
- Inline type `{ fields }` makes the compiler generate `<RpcName>Input` / `<RpcName>Output`. Fields in inline types support annotations:
- Generic return types (`Paginated<T>`) use compile-time monomorphization
- **Streaming**: `stream` keyword before input or output type. Maps to protobuf `stream` in `rpc`.

```
service UserService {
    rpc GetUser(uuid.UUID) -> User;
    rpc CreateUser({
        @validation::Required
        string name = 1;
        @validation::Required
        string email = 2;
    }) -> User;
    rpc UploadUsers(stream User) -> void;                          // client streaming
    rpc ListUsers(void) -> stream User;                            // server streaming
    rpc SyncUsers(stream User) -> stream User;                     // bidirectional
}
```

## Keywords: Pick & Omit

Built-in language keywords (not library features). Shorthand for simple projections.

**Pick** creates a type from a subset of fields:

```
type Sub = Pick<Original, field1, field2>;
```

**Omit** creates a type excluding fields:

```
type Without = Omit<Original, field1, field2>;
type Without2 = Omit<Original, ShapeName>;
```

For shape-based Omit, matching uses name+type pairs. If a referenced shape or field does not exist in the source type, it is a compile error. If the shape exists but none of its fields match, the compiler emits a warning.

Pick and Omit preserve original field numbers. They are syntactic sugar for projections (see below).

## Projections

A projection is a type where some fields have a mapping (`<-`) to fields in other types. It compiles to a separate protobuf message. The mapping metadata is available to plugins for generating conversion code.

A type becomes a projection when it contains at least one `<-` mapping. No special keyword is needed — the `<-` operator is the only difference from a regular type.

```
type Name {
    <type> <field_name> = <field_number> <- <SourceType>.<source_field>;
}
```

Source qualification (`SourceType.field`) is always required.

### Simple mapping

```
type User {
    uuid.UUID id = 1;
    string first_name = 2;
    string last_name = 3;
    string email = 4;
    string password_hash = 5;
    uint64 created_at = 6;
}

type UserAccount {
    uuid.UUID id = 1 <- User.id;
    string first_name = 2 <- User.first_name;
    string last_name = 3 <- User.last_name;
    string email = 4 <- User.email;
    string password_hash = 5 <- User.password_hash;
    uint64 created_at = 6 <- User.created_at;
}
```

### Renaming

```
type UserProfile {
    uuid.UUID user_id = 1 <- User.id;
    string name = 2 <- User.first_name;
}
```

### New fields (no mapping)

Fields without `<-` are new — they exist only in the projection. Plugins do not generate mapping for them.

```
type UserSearch {
    uuid.UUID id = 1 <- User.id;
    string name = 2 <- User.first_name;
    string email = 3 <- User.email;
    string search_text = 4;                // new field, no source mapping
}
```

### Multiple sources

A projection can map fields from multiple source types:

```
type User {
    uuid.UUID id = 1;
    string first_name = 2;
    string email = 3;
}

type UserStats {
    uuid.UUID user_id = 1;
    int64 total_orders = 2;
    int64 total_spent = 3;
}

type UserSettings {
    uuid.UUID user_id = 1;
    string theme = 2;
    string locale = 3;
}

type UserDashboard {
    uuid.UUID id = 1 <- User.id;
    string name = 2 <- User.first_name;
    int64 total_orders = 3 <- UserStats.total_orders;
    int64 total_spent = 4 <- UserStats.total_spent;
    string theme = 5 <- UserSettings.theme;
    string locale = 6 <- UserSettings.locale;
}
```

The compiler infers sources from the `<-` qualifiers.

### Mapping from nested fields

Use dot notation to reach into nested types:

```
type Order {
    int64 id = 1;
    Address billing_address = 2;
}

type Address {
    string street = 1;
    string city = 2;
}

type OrderFlat {
    int64 id = 1 <- Order.id;
    string billing_street = 2 <- Order.billing_address.street;
    string billing_city = 3 <- Order.billing_address.city;
}
```

### Common fields from oneof variants

Use `SourceType.oneof_name.*.field_name` to extract a field that exists in **all** variants of a oneof. The compiler verifies that the field name and type match across all variants.

```
type Payment {
    int64 id = 1;
    oneof method {
        CardPayment card = 2;
        BankPayment bank = 3;
    }
}

type CardPayment {
    string transaction_id = 1;
    int64 amount = 2;
    string card_number = 3;
}

type BankPayment {
    string transaction_id = 1;
    int64 amount = 2;
    string iban = 3;
}

type PaymentFlat {
    int64 id = 1 <- Payment.id;
    string transaction_id = 2 <- Payment.method.*.transaction_id;
    int64 amount = 3 <- Payment.method.*.amount;

    oneof details {
        string card_number = 4 <- Payment.method.card.card_number;
        string iban = 5 <- Payment.method.bank.iban;
    }
}
```

### Grouping fields into a nested type

Define a separate projection and use it as a field type:

```
type User {
    uuid.UUID id = 1;
    string name = 2;
    string street = 3;
    string city = 4;
    string zip = 5;
}

type UserAddress {
    string street = 1 <- User.street;
    string city = 2 <- User.city;
    string zip = 3 <- User.zip;
}

type UserAPI {
    uuid.UUID id = 1 <- User.id;
    string name = 2 <- User.name;
    UserAddress address = 3;
}
```

### Projection chains

A projection can reference another projection as a source:

```
type UserAPI {
    uuid.UUID id = 1 <- User.id;
    string name = 2 <- User.name;
    string email = 3 <- User.email;
    string avatar_url = 4 <- User.avatar_url;
}

type UserCard {
    uuid.UUID id = 1 <- UserAPI.id;
    string name = 2 <- UserAPI.name;
    string avatar_url = 3 <- UserAPI.avatar_url;
}
```

The compiler tracks the full chain: `UserCard.name <- UserAPI.name <- User.name`.

### Oneof → optional fields

```
type Event {
    int64 id = 1;
    oneof payload {
        Click click = 2;
        View view = 3;
    }
}

type EventFlat {
    int64 id = 1 <- Event.id;
    Click? click = 2 <- Event.payload.click;
    View? view = 3 <- Event.payload.view;
}
```

### Fields → oneof

```
type User {
    uuid.UUID id = 1;
    string personal_email = 2;
    string work_email = 3;
}

type UserContact {
    uuid.UUID id = 1 <- User.id;

    oneof email {
        string personal = 2 <- User.personal_email;
        string work = 3 <- User.work_email;
    }
}
```

### Annotations on projections

Annotations work on projections and their fields the same way as on regular types:

```
@database::Table(table_name="user_accounts")
type UserAccount {
    @database::Column(column_name="id", primary_key=true)
    uuid.UUID id = 1 <- User.id;

    @database::Column(column_name="email")
    string email = 2 <- User.email;
}
```

### Full example — one type, multiple representations

```
type User {
    uuid.UUID id = 1;
    string first_name = 2;
    string last_name = 3;
    string personal_email = 4;
    string work_email = 5;
    string street = 6;
    string city = 7;
    string zip = 8;
    string country = 9;
    string bio = 10;
    string avatar_url = 11;
    string password_hash = 12;
    uint64 created_at = 13;
}

// DB: users table
@database::Table(table_name="users")
type UserAccount {
    uuid.UUID id = 1 <- User.id;
    string first_name = 2 <- User.first_name;
    string last_name = 3 <- User.last_name;
    string email = 4 <- User.personal_email;
    string password_hash = 5 <- User.password_hash;
    uint64 created_at = 6 <- User.created_at;
}

// DB: user_profiles table
@database::Table(table_name="user_profiles")
type UserProfile {
    uuid.UUID user_id = 1 <- User.id;
    string bio = 2 <- User.bio;
    string avatar_url = 3 <- User.avatar_url;
    string street = 4 <- User.street;
    string city = 5 <- User.city;
    string zip = 6 <- User.zip;
    string country = 7 <- User.country;
}

// Elasticsearch
type UserSearch {
    uuid.UUID id = 1 <- User.id;
    string name = 2 <- User.first_name;
    string email = 3 <- User.personal_email;
    string bio = 4 <- User.bio;
    string search_text = 5;
}

// API: nested address via separate projection
type UserAddress {
    string street = 1 <- User.street;
    string city = 2 <- User.city;
    string zip = 3 <- User.zip;
    string country = 4 <- User.country;
}

type UserResponse {
    uuid.UUID id = 1 <- User.id;
    string first_name = 2 <- User.first_name;
    string last_name = 3 <- User.last_name;

    oneof email {
        string personal = 4 <- User.personal_email;
        string work = 5 <- User.work_email;
    }

    UserAddress address = 6;
    string avatar_url = 7 <- User.avatar_url;
    string display_name = 8;
}

// Multi-source: dashboard combining User and UserStats
type UserStats {
    uuid.UUID user_id = 1;
    int64 total_orders = 2;
    int64 total_spent = 3;
}

type UserDashboard {
    uuid.UUID id = 1 <- User.id;
    string name = 2 <- User.first_name;
    string email = 3 <- User.personal_email;
    int64 total_orders = 4 <- UserStats.total_orders;
    int64 total_spent = 5 <- UserStats.total_spent;
}
```

### Projection rules

- A projection is a **regular type** that has at least one `<-` mapping. No special keyword needed.
- It compiles to a **separate protobuf message** with its own field numbers.
- Projection field numbers are **independent** from source type field numbers. Renaming or reordering fields in a projection does not affect the source type's wire format, and vice versa.
- Mappings resolve by **field name**, not by field number. If a source field is renamed or removed, all projections referencing it will fail to compile.
- Circular projection references are **forbidden**. The mapping graph must be a DAG — the compiler rejects cycles. Projection chains have no depth limit; the compiler traverses the full DAG to resolve transitive mappings (e.g., `C.x <- B.x <- A.x`).
- If any segment in a mapping path is optional (e.g., `<- Order.billing_address.street` where `billing_address` is `Address?`), the result field is automatically optional.
- `<-` mappings are **compile-time metadata** available to plugins via `OghamCompileRequest`. They do not affect the wire format.
- Source qualification is **always required**: `<- SourceType.field`, not `<- field`.
- Fields without `<-` are new fields with no source mapping.
- Source must be a `type` or another projection (not a `shape` — shapes have no field numbers).
- The compiler validates that mapped source fields exist and have compatible types.
- A projection can have **multiple sources** — the compiler infers them from `<-` qualifiers.
- `SourceType.oneof.*.field` requires the field to exist with the same name and **exactly the same type** in **all** oneof variants. No type widening — `int32` in one variant and `int64` in another is a compile error. If the field is optional in at least one variant, the result is optional.
- `Pick<T, ...>` and `Omit<T, ...>` are shorthand that preserve original field numbers. When applied to a projection, mappings are inherited — the result is a full type that is also a projection. When applied to a regular type (no mappings), the result is a regular type. Example: `type X = Pick<UserAccount, id, email>;` where `UserAccount` is a projection produces a type with `id <- User.id` and `email <- User.personal_email` inherited from `UserAccount`. But `type Y = Pick<User, id, email>;` where `User` is a regular type produces a regular type with no mappings.

## Annotations

### Definition

An annotation is defined in a library with explicit targets and a parameter schema.

```
annotation Name for <target1>|<target2> {
    <type> <param_name>;
    <type>? <param_name> = <default>;
}
```

**Targets**: `shape`, `type`, `field`, `oneof`, `oneof_field`, `enum`, `enum_value`, `service`, `rpc`.

A field is optional if it has `?` or a default value.

### Composition

Annotations can include other annotations — the compiler expands them recursively at compile time. This enables building complex validators from primitives without writing codegen.

```
annotation Email for field {
    validate::Pattern("^[a-zA-Z0-9+_.-]+@[a-zA-Z0-9.-]+$");
    validate::Length(min=3, max=255);
}

annotation RequiredEmail for field {
    validate::Required;
    mylib::Email;
}
```

`@mylib::RequiredEmail` on a field expands to `@validate::Required` + `@validate::Pattern(...)` + `@validate::Length(...)`.

Composition is a general mechanism — not limited to validation. Any annotation can include others:

```
annotation AuditedTable for type {
    string table_name;

    database::Table(table_name=self.table_name);
    database::CreatedAt(column="created_at");
    database::UpdatedAt(column="updated_at");
}

// one annotation instead of three
@mylib::AuditedTable(table_name="users")
type User { ... }
```

Composite annotations can reference their own parameters via `self`:

```
annotation StringId for field {
    int32 length = 20;

    validate::Required;
    validate::Length(exact=self.length);
}
```

Rules:
- Composition targets must be compatible — `Email for field` can only include annotations that also target `field`.
- Circular composition is forbidden.
- Conflicting constraints from composition (e.g., two `@validate::Length` with different `max`) is a compile error.

See [validation.md](validation.md) for the full validation system.

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
| `rpc` | `google.protobuf.MethodOptions` |
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
| `type` with `<-` (projection) | New `message` with mapping metadata in `OghamAnnotation` |
| `oneof` | `oneof` |
| `service` | `service` |
| `rpc` | `rpc` |
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
