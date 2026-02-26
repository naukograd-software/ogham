# Package Management

## Environment and Storage

Ogham uses global storage for dependencies and binaries to avoid duplicating code in every project.

- **`OGHAM_HOME`**: Ogham root directory (default: `~/.ogham`).
- **`OGHAM_BIN`**: Directory for compiled plugin binaries (default: `$OGHAM_HOME/bin`).
- **`OGHAM_CACHE`**: Directory for downloaded package source code (default: `$OGHAM_HOME/pkg/mod`).
- **`OGHAM_PROXY`**: Proxy server URL for package downloads (default: `direct`).

### Proxy Architecture

Ogham can download packages through intermediate mirrors. This is useful for corporate networks, offline builds, and caching public repositories.

Behavior is controlled by the `OGHAM_PROXY` environment variable, which accepts a list of URLs separated by commas or pipes (`|`).

Example:
```bash
export OGHAM_PROXY="https://proxy.company.internal,direct"
```

The `direct` keyword means direct access to the source (for example, cloning from GitHub via git/https). If a proxy returns 404 or 410, Ogham proceeds to the next entry in the list.

**Proxy protocol (REST API):**
The proxy server must serve static files using a defined layout (similar to GOPROXY). For module `github.com/org/db` and version `v1.2.0`:

- `GET /github.com/org/db/@v/v1.2.0.info` - metadata as JSON (version, commit date).
- `GET /github.com/org/db/@v/v1.2.0.mod` - the `ogham.toml` file for that version.
- `GET /github.com/org/db/@v/v1.2.0.zip` - source archive for the module.

### Directory Structure

```
$OGHAM_HOME/
├── bin/                # Compiled plugin binaries (ogham-gen-*)
│   ├── ogham-gen-database@v2.0.0
│   └── ogham-gen-grpc@v1.0.3
└── pkg/
    └── mod/            # Module source code (read-only cache)
        └── github.com/
            └── org/
                └── database@v2.0.0/
                    ├── ogham.toml
                    └── ...
```

## Module System

Works similarly to Go. A module is the root unit identified by a URL path. A package is a directory inside a module.

```
myproject/
├── ogham.toml          # module manifest
├── ogham.lock          # lock file (generated automatically)
├── models/
│   ├── user.ogham      # package models
│   └── order.ogham     # package models
└── api/
    └── contracts.ogham # package api
```

Files in the same directory belong to one package and can reference each other's types directly without `import`. Dependency packages are stored globally in `OGHAM_CACHE`.

## Import

```
import uuid;                          // standard library
import github.com/org/database;       // external dependency
import github.com/org/database/pg;    // external dependency subpackage
```

The last path segment becomes the name used in code:

```
import github.com/org/database;

@database::Table(table_name="users")
type User { ... }
```

Use aliases to resolve naming conflicts:

```
import github.com/org/database as mydb;
import github.com/other/database as otherdb;
```

## Visibility

- **Uppercase** - exported from the package (`User`, `OrderStatus`, `Table`)
- **lowercase** - package-internal only (`userHelper`, `internalShape`)

## Manifest: ogham.toml

The manifest defines module metadata, dependencies, features, and plugin configuration.

```toml
[package]
name = "github.com/org/project"
version = "1.2.0"
description = "E-commerce schema definitions"
license = "MIT"
ogham = ">=0.1.0"                     # minimum compiler version

[dependencies]
"github.com/ogham/std" = "^1.0.0"
"github.com/ogham/uuid" = "^1.0.0"
"github.com/org/database" = { version = "^2.0.0", features = ["postgres", "go"] }

[features]
default = ["grpc-api"]
grpc-api = []
rest-api = []
admin-panel = ["grpc-api"]            # admin-panel enables grpc-api

# Optional dependencies activated through features
[features.dependencies]
grpc-api = { "github.com/org/grpc-gen" = "^1.0.0" }
rest-api = { "github.com/org/rest-gen" = "^1.0.0" }
```

### Section [package]

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Full module path (URL-based, as in Go) |
| `version` | yes | Semver version |
| `description` | no | Description |
| `license` | no | SPDX license identifier |
| `ogham` | no | Minimum compatible compiler version |

### Section [dependencies]

Versions follow semver ranges:

| Syntax | Meaning |
|--------|---------|
| `"^1.2.0"` | `>=1.2.0, <2.0.0` |
| `"~1.2.0"` | `>=1.2.0, <1.3.0` |
| `"=1.2.0"` | Exact version |
| `">=1.0.0, <3.0.0"` | Explicit range |

### Section [features]

Similar to Cargo. A feature is a named flag that:
- Enables optional dependencies
- Is passed to plugins as context
- Can activate other features (transitively)
- Can activate dependency features (via `"dep/feature"` syntax)

`default` defines features enabled by default. Consumers can override this:

```toml
[dependencies]
"github.com/org/project" = { version = "^1.0.0", default-features = false, features = ["rest-api"] }
```

### Feature Requirements (propagation)

A plugin can require specific features from its dependencies. As in Cargo, this uses **additive unification**: all requested features are enabled.

```toml
# ogham.toml of plugin ogham-gen-go-pgx
[package]
name = "github.com/org/go-pgx"

[dependencies]
# Hard requirement: database MUST have "go" feature
"github.com/org/database" = { version = "^2.0.0", features = ["go"] }

[features]
default = ["go"]
go = ["github.com/org/database/go"]    # our "go" feature enables "go" on database
```

If two plugins require different features on the same dependency, all features are enabled (union). Features must be additive: enabling one should not break code that works without it.

## Lock File: ogham.lock

Generated automatically. Contains the resolved dependency graph with exact versions and checksums. Similar to `go.sum`. Must be committed to the repository.

```toml
[[lock]]
name = "github.com/ogham/uuid"
version = "1.0.3"
checksum = "sha256:abc123..."

[[lock]]
name = "github.com/org/database"
version = "2.1.0"
checksum = "sha256:def456..."
dependencies = ["github.com/ogham/std@1.0.0"]
features = ["postgres", "go"]
```

## CLI Commands

CLI commands are documented in a separate file: [cmd.md](cmd.md).

---

# Plugin System

A plugin is a module that defines annotations and provides code generation and/or validation.

## Plugin Naming Convention

Plugin binaries follow the `ogham-gen-<name>` convention (similar to `protoc-gen-*`):

- `github.com/org/database` -> `ogham-gen-database`
- `github.com/org/grpc` -> `ogham-gen-grpc`
- `github.com/org/go-pgx` -> `ogham-gen-go-pgx`

The binary name is derived automatically from the last module path segment with the `ogham-gen-` prefix.

**Discovery**: the compiler searches for the binary in this order:
1. `$OGHAM_BIN/ogham-gen-<name>@<version>` - specific version
2. `$PATH` - globally installed plugins

## Plugin Lifecycle & Distribution

1. **Install**: `ogham get` or `ogham install` downloads source code into `$OGHAM_CACHE`.
2. **Build**: `ogham` runs the `build` command from `[plugin]` automatically. Output: `$OGHAM_BIN/ogham-gen-<name>@<version>`. **build is required** for stdio plugins.
3. **Invoke**: the compiler finds `ogham-gen-<name>` and runs it via stdio or connects via gRPC.

## Plugin Protocol

### stdio

The compiler starts `ogham-gen-<name>` as a process and communicates via stdin/stdout.

```
ogham compile -> stdin: OghamCompileRequest (protobuf) -> [ogham-gen-*] -> stdout: OghamCompileResponse (protobuf)
```

In proto mode, the plugin receives standard `google.protobuf.compiler.CodeGeneratorRequest`:

```
ogham compile --proto -> stdin: CodeGeneratorRequest (protobuf) -> [ogham-gen-*] -> stdout: CodeGeneratorResponse (protobuf)
```

### gRPC

The compiler connects to a running plugin gRPC service.

```
ogham compile -> gRPC call: PluginService.Generate(OghamCompileRequest) -> OghamCompileResponse
```

gRPC is useful for:
- Heavy plugins with long cold start time
- Plugin-as-a-service setups (shared in CI)
- Watch mode (plugin keeps state between compilations)

## Plugin Manifest

If a module is a plugin, `ogham.toml` includes a `[plugin]` section:

```toml
[package]
name = "github.com/org/database"
version = "2.0.0"
description = "Database codegen plugin for Ogham"

[plugin]
protocol = "stdio"                    # "stdio" | "grpc"
build = "go build -o ogham-gen-database ./cmd"  # REQUIRED for stdio

# For grpc:
# protocol = "grpc"
# build = "go build -o ogham-gen-database ./cmd"  # required - builds gRPC server
# address = "localhost:50051"          # default address (overridable by consumer)

# What the plugin provides
provides = ["annotations", "codegen"] # "annotations" | "codegen" | "validation"

# Code generation target languages
targets = ["go", "typescript", "rust"]

[features]
default = ["ogham"]
ogham = []                            # receives OghamCompileRequest (native AST)
proto = []                            # receives CodeGeneratorRequest (protobuf standard)

[plugin.options]
# Options passed to plugin invocation (configured by consumer)
output_dir = { type = "string", default = "gen/" }
orm = { type = "string", default = "sqlc", enum = ["sqlc", "sqlx", "gorm"] }
```

### Section [plugin]

| Field | Required | Description |
|-------|----------|-------------|
| `protocol` | yes | Invocation protocol: `stdio` or `grpc` |
| `build` | **yes** | Plugin build command. Required - every plugin must define how it is built |
| `address` | grpc only | gRPC service address (`host:port`) |
| `provides` | yes | Provided capabilities: `annotations`, `codegen`, `validation` |
| `targets` | codegen only | Code generation target languages |

### Feature: `proto`

A plugin can support two input modes via features:

| Feature | Input | Description |
|---------|-------|-------------|
| `ogham` | `OghamCompileRequest` | Native Ogham AST - typed and complete |
| `proto` | `CodeGeneratorRequest` | Standard protobuf input - protoc ecosystem compatibility |

A plugin with the `proto` feature can work with `.proto` files, so it can participate in the proto pipeline alongside standard `protoc-gen-*` plugins.

### Section [plugin.options]

Typed options that a consumer can override in `ogham.toml`:

```toml
# In consumer ogham.toml
[generate.options."github.com/org/database"]
output_dir = "src/generated/"
orm = "sqlx"
```

## Generation Modes

### Native Mode (default)

The compiler parses `.ogham` files and sends `OghamCompileRequest` to plugins:

```
*.ogham -> ogham compiler -> OghamCompileRequest -> ogham-gen-* plugins -> generated code
```

Only `ogham-gen-*` plugins are used.

### Proto Mode

The compiler first generates `.proto` files from `.ogham`, then runs plugins:

```
*.ogham -> ogham compiler -> *.proto (with OghamAnnotation options)
                              ↓
                    plugin invocation:
                    ├── ogham-gen-* (with proto feature, receive CodeGeneratorRequest)
                    ├── protoc-gen-* (standard protobuf plugins)
                    └── generated code
```

In proto mode:
- Annotations are serialized as `OghamAnnotation { name, google.protobuf.Struct }` (see `ogham/options.proto`)
- Standard `protoc-gen-*` plugins are allowed (`protoc-gen-go`, `protoc-gen-go-grpc`, `protoc-gen-grpc-gateway`, etc.)
- `protoc-gen-*` plugins are discovered via `$PATH` (standard protobuf discovery)
- All plugins receive the same `CodeGeneratorRequest` and are independent from each other

## Consumer Configuration

Consumers define which plugins to use and with what parameters in their `ogham.toml`:

```toml
[package]
name = "github.com/myteam/myproject"
version = "0.1.0"

[dependencies]
"github.com/ogham/std" = "^1.0.0"
"github.com/ogham/uuid" = "^1.0.0"
"github.com/org/database" = { version = "^2.0.0", features = ["postgres", "go"] }
"github.com/org/grpc-gen" = { version = "^1.0.0", features = ["go"] }

[generate]
mode = "proto"                        # "proto" | "native" (default: "native")

# Which plugins to run during compilation and in what order
plugins = [
    # ogham plugins (ogham-gen-*)
    "github.com/org/database",
    "github.com/org/grpc-gen",
    # standard protobuf plugins (protoc-gen-*) - proto mode only
    "protoc-gen-go",
    "protoc-gen-go-grpc",
]

[generate.options."github.com/org/database"]
output_dir = "internal/db/gen/"
orm = "sqlx"

[generate.options."protoc-gen-go"]
output_dir = "internal/pb/"
```

## Local Development

Local paths are supported for plugin development. In this mode, `ogham` watches plugin source changes.

### Local Path Dependencies

You can point a dependency to a local plugin directory in `ogham.toml`:

```toml
[dependencies]
"my-plugin" = { path = "../plugins/my-plugin" }
```

The compiler rebuilds the plugin on the next `ogham compile` run if files in that directory changed. Binaries for local `path` dependencies are executed directly from their build location without polluting `OGHAM_BIN`.

### Bootstrapping

Create a new plugin scaffold in the current directory:

```bash
ogham init --plugin <name>
```

This generates a base `ogham.toml` with a `[plugin]` section, a `build` command, and file layout. The binary name will be `ogham-gen-<name>`.

## Remote Plugins (gRPC)

Plugins can run as remote services.

### Serving a Plugin

If a plugin supports `grpc`, it can be started as a server:

```bash
ogham serve --plugin <name> --address :50051
```

The Ogham compiler can connect to remote plugins:

```toml
# In consumer ogham.toml
[dependencies]
"remote-plugin" = { version = "^1.0.0", address = "grpc.prod.internal:50051" }
```

This enables shared plugin usage in CI/CD or distributed teams without installing binaries on every machine.

## Full Example

Project: Go + PostgreSQL + gRPC with proto mode.

```
myproject/
├── ogham.toml
├── ogham.lock
├── schemas/
│   ├── models.ogham        # package schemas - types and enums
│   └── api.ogham           # package schemas - services and contracts
├── internal/
│   ├── db/gen/             # <- output from ogham-gen-database
│   ├── api/gen/            # <- output from ogham-gen-grpc
│   └── pb/                 # <- output from protoc-gen-go + protoc-gen-go-grpc
```

```toml
# ogham.toml
[package]
name = "github.com/myteam/myproject"
version = "0.1.0"
ogham = ">=0.1.0"

[dependencies]
"github.com/ogham/std" = "^1.0.0"
"github.com/ogham/uuid" = "^1.0.0"
"github.com/org/database" = { version = "^2.0.0", features = ["postgres", "go"] }
"github.com/org/grpc-gen" = { version = "^1.0.0", features = ["go"] }

[features]
default = ["proto"]
proto = []

[generate]
mode = "proto"
plugins = [
    # ogham plugins - read OghamAnnotation options from .proto
    "github.com/org/database",
    "github.com/org/grpc-gen",
    # standard protobuf plugins - generate Go code from .proto
    "protoc-gen-go",
    "protoc-gen-go-grpc",
]

[generate.options."github.com/org/database"]
output_dir = "internal/db/gen/"
orm = "sqlx"

[generate.options."github.com/org/grpc-gen"]
output_dir = "internal/api/gen/"

[generate.options."protoc-gen-go"]
output_dir = "internal/pb/"

[generate.options."protoc-gen-go-grpc"]
output_dir = "internal/pb/"
```

```bash
ogham compile                         # parses schemas/, generates .proto, invokes all plugins
ogham compile --plugin=database       # only one plugin
ogham compile --target=go             # only for a specific target
ogham compile --mode=native           # override: native mode instead of proto
```

### Pipeline in proto mode

```
schemas/*.ogham
    ↓ ogham compiler (parse + generate .proto)
schemas/*.proto  (contain OghamAnnotation options in ogham/options.proto)
    ↓ parallel plugin invocation (all receive the same CodeGeneratorRequest)
    ├── ogham-gen-database    → internal/db/gen/     (reads OghamAnnotation "database::Table", etc.)
    ├── ogham-gen-grpc        → internal/api/gen/    (reads OghamAnnotation "grpc::*")
    ├── protoc-gen-go         → internal/pb/         (generates Go structs)
    └── protoc-gen-go-grpc    → internal/pb/         (generates gRPC stubs)
```

## Feature Dependency Example

Plugin `ogham-gen-go-pgx` depends on `database` and requires its `go` feature:

```toml
# ogham.toml of plugin ogham-gen-go-pgx
[package]
name = "github.com/org/go-pgx"
version = "1.0.0"

[dependencies]
"github.com/org/database" = { version = "^2.0.0", features = ["go", "postgres"] }

[features]
default = ["ogham", "proto"]
ogham = []
proto = []
go = ["github.com/org/database/go"]    # our "go" feature -> enables "go" on database

[plugin]
build = "go build -o ogham-gen-go-pgx ./cmd"
protocol = "stdio"
provides = ["codegen"]
targets = ["go"]
```

When a consumer adds `go-pgx`, the `go` feature on `database` is enabled automatically:

```toml
# consumer ogham.toml
[dependencies]
"github.com/org/go-pgx" = "^1.0.0"
# database automatically gets features = ["go", "postgres"] via go-pgx
```
