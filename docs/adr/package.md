# Package Management

## Environment and Storage

Ogham uses global storage for dependencies and binaries.

- **`OGHAM_HOME`**: Ogham root directory (default: `~/.ogham`).
- **`OGHAM_BIN`**: Compiled plugin binaries (default: `$OGHAM_HOME/bin`).
- **`OGHAM_CACHE`**: Downloaded package source code (default: `$OGHAM_HOME/pkg/mod`).
- **`OGHAM_PROXY`**: Proxy server URL(s) for package downloads (default: `direct`). Supports chaining: `https://internal.proxy,https://proxy.ogham.dev,direct`.

### Proxy Protocol

REST API layout (similar to GOPROXY). For module `github.com/org/db` version `v1.2.0`:

- `GET /github.com/org/db/@v/v1.2.0.info` — metadata as JSON (version, commit date).
- `GET /github.com/org/db/@v/v1.2.0.mod` — the `ogham.mod.yaml` file.
- `GET /github.com/org/db/@v/v1.2.0.zip` — source archive.

### Directory Structure

```
$OGHAM_HOME/
├── bin/                # Compiled plugin binaries (ogham-gen-*)
│   ├── ogham-gen-database@v2.0.0
│   └── ogham-gen-go@v1.0.3
├── git/                # Git repository cache
│   ├── db/             # Bare clones (shared across projects)
│   └── checkouts/      # Revision-specific checkouts
└── pkg/
    └── mod/            # Module source code (read-only cache)
        └── github.com/
            └── org/
                └── database@v2.0.0/
                    ├── ogham.mod.yaml
                    └── ...
```

## Module System

A module is the root unit identified by a URL path. A package is a directory inside a module.

```
myproject/
├── ogham.mod.yaml      # module manifest
├── ogham.gen.yaml      # generation config (optional)
├── models/
│   ├── user.ogham
│   └── order.ogham
└── api/
    └── contracts.ogham
```

Files in the same directory belong to one package and can reference each other's types directly without `import`.

## Two Files

### `ogham.mod.yaml` — Module Manifest

Declares who the module is, what it depends on, and (for plugins) how to build.

Present in every module. Travels with the package when published.

### `ogham.gen.yaml` — Generation Config

Declares how to generate code: which plugins to run, where to put output.

Present only in projects that generate code. Not published — each consumer has their own.

## Import

```
import uuid;                          // standard library
import github.com/org/database;       // external dependency
import github.com/org/database/pg;    // subpackage
```

Last path segment becomes the name:

```
import github.com/org/database;

@database::Table(table_name="users")
type User { ... }
```

Aliases resolve conflicts:

```
import github.com/org/database as mydb;
import github.com/other/database as otherdb;
```

## Visibility

- **Uppercase** — exported (`User`, `Table`)
- **lowercase** — package-private (`userHelper`)

## ogham.mod.yaml

### Schema Package

```yaml
module: github.com/myteam/myproject
ogham: ">= 0.1.0"
version: 0.1.0
description: E-commerce schema definitions
license: MIT

require:
  github.com/ogham/std: ^1.0.0
  github.com/ogham/uuid: ^1.0.0
  github.com/org/database: ^2.0.0       # provides annotations, used in .ogham code
  github.com/org/go: ^1.0.0             # codegen plugin

replace:
  github.com/org/database:
    path: ../database-fork
```

### Plugin Package

A plugin is a module that also has a `plugin` section:

```yaml
module: github.com/org/database
version: 2.0.0
description: Database codegen plugin for Ogham

require:
  github.com/ogham/std: ^1.0.0

plugin:
  build: go build -o ogham-gen-database ./cmd
```

### Fields

#### Module fields

| Field | Required | Description |
|-------|----------|-------------|
| `module` | yes | Full module path (URL-based) |
| `version` | yes | Semver version |
| `ogham` | no | Minimum compiler version |
| `description` | no | Description |
| `license` | no | SPDX license identifier |

#### `require` section

Dependencies used in `.ogham` code (schema packages, annotation providers) and codegen plugins.

Versions follow semver ranges:

| Syntax | Meaning |
|--------|---------|
| `^1.2.0` | `>=1.2.0, <2.0.0` |
| `~1.2.0` | `>=1.2.0, <1.3.0` |
| `=1.2.0` | Exact version |
| `>=1.0.0, <3.0.0` | Explicit range |

##### Git Dependencies

```yaml
require:
  github.com/org/timestamps:
    git: https://github.com/org/timestamps.git
    tag: v1.0.0

  github.com/org/experimental:
    git: https://github.com/org/experimental.git
    branch: next

  github.com/org/debug:
    git: https://github.com/org/debug.git
    rev: a1b2c3d
```

Git dependencies are cached in `$OGHAM_HOME/git/`. `git` and version range are mutually exclusive.

##### Path Dependencies

```yaml
require:
  my-plugin:
    path: ../plugins/my-plugin
```

For local development only. `ogham publish` rejects modules with path dependencies.

#### `replace` section

Override any dependency with a local path or git fork:

```yaml
replace:
  github.com/org/database:
    path: ../database-fork

  github.com/ogham/uuid:
    git: https://github.com/me/uuid.git
    branch: fix-parsing
```

Rules:
- Only applies in the root module. Replace in transitive dependencies is ignored.
- `ogham publish` rejects modules with active `replace` entries.

#### `plugin` section

Present only in plugin modules.

| Field | Required | Description |
|-------|----------|-------------|
| `build` | yes | Shell command to build the plugin binary |

The build command must produce a binary named `ogham-gen-<last-segment-of-module-path>`. All plugins receive `OghamCompileRequest` via stdin.

## ogham.gen.yaml

Present only in projects that generate code.

```yaml
generate:
  plugins:
    # ogham plugin from require — compiler builds binary automatically
    - name: github.com/org/database
      out: internal/db/gen/
      opts:
        orm: sqlx

    # ogham plugin from require
    - name: github.com/org/go
      out: internal/models/

    # ogham plugin via gRPC
    - name: github.com/org/go-pgx
      grpc: localhost:50051
      out: internal/db/gen/

    # external binary — relative to project root
    - path: ./tools/my-custom-plugin
      out: gen/
```

### Plugin entry fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | * | Full module path from `require` in `ogham.mod.yaml` |
| `path` | * | External binary: full path, relative path, or short name (searched in `$PATH`) |
| `grpc` | no | gRPC address (`host:port`). Overrides stdio invocation for `name` plugins |
| `out` | yes | Output directory for generated files |
| `opts` | no | Key-value options passed to the plugin |

`name` and `path` are mutually exclusive — one must be specified.

### Plugin resolution for `name`

1. `$OGHAM_BIN/ogham-gen-<name>@<version>` — pre-built binary
2. If not found, compiler runs `build` from the plugin's `ogham.mod.yaml`
3. Built binary cached in `$OGHAM_BIN`

### Plugin resolution for `path`

1. If absolute or relative path — use directly
2. If short name — search in `$PATH`

### Execution order

Plugins execute in the order listed in `ogham.gen.yaml`. User controls the order.

### Generation pipeline

```
*.ogham → ogham compiler → OghamCompileRequest → ogham-gen-* plugins → generated code
```

All plugins receive `OghamCompileRequest` via stdin and respond with `OghamCompileResponse` via stdout. See [wire.md](wire.md) for serialization formats.

## Dependency Resolution

Ogham uses **Minimal Version Selection (MVS)**:

1. Collect all version requirements across the dependency graph.
2. For each package, select the **minimum** version that satisfies all requirements.
3. Each package resolves to exactly **one version**. Multiple versions of the same package are not allowed — schema type identity must be unambiguous.

### Why MVS?

- **Deterministic** — same `ogham.mod.yaml` files → same resolution, no lock file needed.
- **No backtracking** — single pass, simple to implement.
- **Conservative** — `ogham update` only bumps what you ask for.
- **Correct for schemas** — minimizes surprise version changes.

## CLI Commands

```bash
# Dependencies
ogham get github.com/org/database             # add dependency
ogham get github.com/org/database@2.1.0       # specific version
ogham install                                  # fetch all dependencies
ogham update                                   # update versions
ogham vendor                                   # copy to vendor/

# Generation
ogham generate                                 # run all plugins from ogham.gen.yaml
ogham generate --plugin=database               # run single plugin

# Proto export
ogham proto export ./proto/                    # export .proto files for external toolchains

# Plugin development
ogham init --plugin <name>                     # scaffold a new plugin

# Remote plugins
ogham serve --plugin <name> --address :50051   # serve plugin as gRPC
```

## Full Example

Project: Go + PostgreSQL.

```
myproject/
├── ogham.mod.yaml
├── ogham.gen.yaml
├── schemas/
│   ├── models.ogham
│   └── api.ogham
├── internal/
│   ├── models/         ← ogham-gen-go
│   └── db/gen/         ← ogham-gen-database
```

```yaml
# ogham.mod.yaml
module: github.com/myteam/myproject
version: 0.1.0
ogham: ">= 0.1.0"

require:
  github.com/ogham/std: ^1.0.0
  github.com/ogham/uuid: ^1.0.0
  github.com/org/database: ^2.0.0
  github.com/org/go: ^1.0.0
```

```yaml
# ogham.gen.yaml
generate:
  plugins:
    - name: github.com/org/go
      out: internal/models/
    - name: github.com/org/database
      out: internal/db/gen/
      opts:
        orm: sqlx
```

```bash
ogham generate
```

```
schemas/*.ogham
    ↓ ogham compiler → OghamCompileRequest
    ↓ plugins in listed order
    ├── ogham-gen-go          → internal/models/   (Go structs + protowire + protojson)
    └── ogham-gen-database    → internal/db/gen/   (SQL mappings, queries)
```
