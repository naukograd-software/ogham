# Validation

## Overview

Validation in Ogham is built on **annotation composition**. A small set of primitive validation annotations is provided by the standard `ogham-validate` plugin. Complex validators are composed from primitives — no code generation required from plugin authors.

## Primitive Annotations

Provided by `ogham-validate` plugin. The compiler understands these natively for type checking and LSP support.

### `@validate::Required`

Field must be set to a non-zero value.

```
@validate::Required
string email = 1;
```

**Target**: `field`

### `@validate::Length`

Constrains length of `string`, `bytes`, `[]T`, and `map<K,V>`.

```
@validate::Length(min=1, max=255)
string name = 1;

@validate::Length(exact=16)
[]byte uuid = 2;

@validate::Length(min=1)
[]Order orders = 3;
```

| Param | Type | Description |
|-------|------|-------------|
| `min` | `int32?` | Minimum length (inclusive) |
| `max` | `int32?` | Maximum length (inclusive) |
| `exact` | `int32?` | Exact length (shorthand for min=N, max=N) |

**Target**: `field` (string, bytes, repeated, map)

### `@validate::Range`

Constrains numeric value.

```
@validate::Range(min=0, max=150)
int32 age = 1;

@validate::Range(min=0.0, exclusive_min=true)
double price = 2;
```

| Param | Type | Description |
|-------|------|-------------|
| `min` | `double?` | Minimum value |
| `max` | `double?` | Maximum value |
| `exclusive_min` | `bool` | Exclude min from range (default: false) |
| `exclusive_max` | `bool` | Exclude max from range (default: false) |

**Target**: `field` (numeric types)

### `@validate::Pattern`

String must match regex.

```
@validate::Pattern(regex="^[a-z][a-z0-9_]*$")
string username = 1;
```

| Param | Type | Description |
|-------|------|-------------|
| `regex` | `string` | RE2 regex pattern |

**Target**: `field` (string)

### `@validate::In`

Value must be one of the listed values.

```
@validate::In(values=["active", "inactive", "pending"])
string status = 1;
```

| Param | Type | Description |
|-------|------|-------------|
| `values` | `[]T` | Allowed values |

**Target**: `field` (string, numeric, enum)

### `@validate::NotIn`

Value must not be one of the listed values.

```
@validate::NotIn(values=[0])
int32 category_id = 1;
```

| Param | Type | Description |
|-------|------|-------------|
| `values` | `[]T` | Disallowed values |

**Target**: `field` (string, numeric, enum)

### `@validate::Unique`

Elements of a repeated field must be unique.

```
@validate::Unique
[]string tags = 1;
```

**Target**: `field` (repeated)

## Annotation Composition

Any annotation can include other annotations. This is how complex validators are built from primitives — like shapes compose fields.

### Defining a composite annotation

```
annotation Email for field {
    validate::Pattern("^[a-zA-Z0-9+_.-]+@[a-zA-Z0-9.-]+$");
    validate::Length(min=3, max=255);
}
```

`@mylib::Email` applied to a field is **expanded** by the compiler into:
- `@validate::Pattern(regex="^[a-zA-Z0-9+_.-]+@[a-zA-Z0-9.-]+$")`
- `@validate::Length(min=3, max=255)`

The master validation plugin sees only the primitives.

### Composite with parameters

A composite annotation can expose parameters that are passed to inner annotations:

```
annotation StringId for field {
    int32 length = 20;

    validate::Required;
    validate::Pattern("^[a-zA-Z0-9]+$");
    validate::Length(exact=self.length);
}
```

Usage:

```
@mylib::StringId(length=36)
string id = 1;
```

Expands to:
- `@validate::Required`
- `@validate::Pattern(regex="^[a-zA-Z0-9]+$")`
- `@validate::Length(exact=36)`

### Multi-level composition

Composite annotations can include other composites:

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

The compiler recursively expands all compositions down to primitives.

### Composition rules

- Annotations can include other annotations as composition members.
- Composition is expanded at compile time — the master plugin only sees primitives.
- `self.<param>` references the composite annotation's own parameters.
- Circular composition is forbidden — the compiler rejects cycles.
- Composition does not change the annotation's target — `Email for field` can only include annotations that also target `field`.
- If composition produces conflicting constraints (e.g., two `@validate::Length` with different `max`), the compiler reports an error.

## How Validation Code Generation Works

1. **Compile time**: the compiler expands all composite annotations to primitives and includes them in `OghamCompileRequest`.
2. **Plugin time**: the `ogham-gen-go-validate` plugin (or any language-specific variant) reads primitive annotations from the request and generates a `Validate()` function.
3. **Third-party annotations**: a plugin like `mylib` defines composite annotations in `.ogham` files. No code generation needed from `mylib` — the master validator plugin handles everything.

```
// OghamCompileRequest (simplified)
{
  "types": [{
    "name": "User",
    "fields": [{
      "name": "email",
      "number": 1,
      "type": "string",
      "annotations": [
        { "name": "validate::Required", "params": {} },
        { "name": "validate::Pattern", "params": { "regex": "^[a-zA-Z0-9+_.-]+@[a-zA-Z0-9.-]+$" } },
        { "name": "validate::Length", "params": { "min": 3, "max": 255 } }
      ]
    }]
  }]
}
```

The master plugin doesn't need to know about `@mylib::Email` — it only sees `@validate::Pattern` and `@validate::Length`.

## Extending Validation

To create a new validator library:

1. Create a package with composite annotations:

```
// package: github.com/myorg/validators
package validators;

import validate;

annotation Email for field {
    validate::Pattern("^[a-zA-Z0-9+_.-]+@[a-zA-Z0-9.-]+$");
    validate::Length(min=3, max=255);
}

annotation UUID for field {
    validate::Pattern("^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$");
    validate::Length(exact=36);
}

annotation Slug for field {
    validate::Pattern("^[a-z0-9]+(-[a-z0-9]+)*$");
    validate::Length(min=1, max=128);
}
```

2. Publish the package. No plugin binary needed — it's just `.ogham` files with annotation definitions.

3. Consumers use it:

```
import github.com/myorg/validators;

type Article {
    @validators::UUID
    string id = 1;

    @validators::Slug
    string slug = 2;

    @validate::Required
    @validate::Length(min=1, max=1000)
    string title = 3;
}
```

The compiler expands `@validators::UUID` and `@validators::Slug` to primitives. The master validation plugin generates code for all of them.

## Proto Mapping

Validation annotations are included in `OghamCompileRequest` (after composition expansion). The validation plugin reads them and generates code. When using `ogham proto export`, validation annotations are serialized as `OghamAnnotation` in proto options.
