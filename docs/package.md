# Package Management

## Environment and Storage

Ogham использует глобальное хранилище для зависимостей и бинарников, что позволяет избежать дублирования кода в каждом проекте.

- **`OGHAM_HOME`**: Корневая директория Ogham (по умолчанию `~/.ogham`).
- **`OGHAM_BIN`**: Директория для скомпилированных плагинов (по умолчанию `$OGHAM_HOME/bin`).
- **`OGHAM_CACHE`**: Директория для скачанных исходных кодов пакетов (по умолчанию `$OGHAM_HOME/pkg/mod`).
- **`OGHAM_PROXY`**: URL прокси-сервера для загрузки пакетов (по умолчанию `direct`).

### Proxy Architecture

Ogham поддерживает загрузку пакетов через промежуточные зеркала. Это полезно для корпоративных сетей, оффлайн-сборок или кэширования публичных репозиториев.

Поведение управляется переменной окружения `OGHAM_PROXY`, которая принимает список URL, разделенных запятыми или пайпами (`|`).

Пример:
```bash
export OGHAM_PROXY="https://proxy.company.internal,direct"
```

Ключевое слово `direct` означает прямое обращение к источнику (например, клонирование с GitHub через git/https). Если прокси возвращает 404 или 410, Ogham переходит к следующему элементу в списке.

**Протокол прокси (REST API):**
Прокси-сервер должен отдавать статические файлы по определенной структуре (аналогично GOPROXY). Для модуля `github.com/org/db` и версии `v1.2.0`:

- `GET /github.com/org/db/@v/v1.2.0.info` — метаданные в JSON (версия, дата коммита).
- `GET /github.com/org/db/@v/v1.2.0.mod` — файл `ogham.toml` этой версии.
- `GET /github.com/org/db/@v/v1.2.0.zip` — архив с исходным кодом модуля.

### Directory Structure

```
$OGHAM_HOME/
├── bin/                # Скомпилированные бинарники плагинов (ogham-gen-*)
│   ├── ogham-gen-database@v2.0.0
│   └── ogham-gen-grpc@v1.0.3
└── pkg/
    └── mod/            # Исходный код модулей (read-only cache)
        └── github.com/
            └── org/
                └── database@v2.0.0/
                    ├── ogham.toml
                    └── ...
```

## Module System

Работает как в Go. Модуль — корневая единица, идентифицируемая URL-путём. Пакет — директория внутри модуля.

```
myproject/
├── ogham.toml          # манифест модуля
├── ogham.lock          # lock-file (генерируется автоматически)
├── models/
│   ├── user.ogham      # package models
│   └── order.ogham     # package models
└── api/
    └── contracts.ogham # package api
```

Файлы в одной директории — один пакет. Они видят типы друг друга напрямую без import. Пакеты зависимостей хранятся глобально в `OGHAM_CACHE`.

## Import

```
import uuid;                          // стандартная библиотека
import github.com/org/database;       // внешняя зависимость
import github.com/org/database/pg;    // подпакет внешней зависимости
```

Последний сегмент пути — имя для использования в коде:

```
import github.com/org/database;

@database::Table(table_name="users")
type User { ... }
```

Alias при конфликте имён:

```
import github.com/org/database as mydb;
import github.com/other/database as otherdb;
```

## Visibility

- **Uppercase** — экспортируется из пакета (`User`, `OrderStatus`, `Table`)
- **lowercase** — только внутри пакета (`userHelper`, `internalShape`)

## Manifest: ogham.toml

Манифест описывает модуль: метаданные, зависимости, features и конфигурацию плагина.

```toml
[package]
name = "github.com/org/project"
version = "1.2.0"
description = "E-commerce schema definitions"
license = "MIT"
ogham = ">=0.1.0"                     # минимальная версия компилятора

[dependencies]
"github.com/ogham/std" = "^1.0.0"
"github.com/ogham/uuid" = "^1.0.0"
"github.com/org/database" = { version = "^2.0.0", features = ["postgres", "go"] }

[features]
default = ["grpc-api"]
grpc-api = []
rest-api = []
admin-panel = ["grpc-api"]            # admin-panel включает grpc-api

# Опциональные зависимости активируемые через features
[features.dependencies]
grpc-api = { "github.com/org/grpc-gen" = "^1.0.0" }
rest-api = { "github.com/org/rest-gen" = "^1.0.0" }
```

### Секция [package]

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Полный путь модуля (URL-based, как в Go) |
| `version` | yes | Semver версия |
| `description` | no | Описание |
| `license` | no | SPDX идентификатор лицензии |
| `ogham` | no | Минимальная совместимая версия компилятора |

### Секция [dependencies]

Версии следуют semver с диапазонами:

| Syntax | Meaning |
|--------|---------|
| `"^1.2.0"` | `>=1.2.0, <2.0.0` |
| `"~1.2.0"` | `>=1.2.0, <1.3.0` |
| `"=1.2.0"` | Точная версия |
| `">=1.0.0, <3.0.0"` | Явный диапазон |

### Секция [features]

Как в Cargo. Feature — именованный флаг, который:
- Включает опциональные зависимости
- Передаётся плагинам как контекст
- Может активировать другие features (транзитивно)
- Может активировать features зависимостей (`"dep/feature"` синтаксис)

`default` — features включённые по умолчанию. Потребитель может переопределить:

```toml
[dependencies]
"github.com/org/project" = { version = "^1.0.0", default-features = false, features = ["rest-api"] }
```

### Feature Requirements (propagation)

Плагин может требовать определённые features от своих зависимостей. Это работает как в Cargo — **аддитивная унификация**: все запрошенные features включаются.

```toml
# ogham.toml плагина ogham-gen-go-pgx
[package]
name = "github.com/org/go-pgx"

[dependencies]
# Hard requirement: database MUST have "go" feature
"github.com/org/database" = { version = "^2.0.0", features = ["go"] }

[features]
default = ["go"]
go = ["github.com/org/database/go"]    # наша фича "go" включает "go" у database
```

Если два плагина требуют разные features от одной зависимости — все features включаются (union). Features должны быть аддитивными: включение feature не должно ломать код без него.

## Lock File: ogham.lock

Генерируется автоматически. Содержит разрешённое дерево зависимостей с точными версиями и checksums. Как `go.sum`. Коммитится в репозиторий.

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

```bash
ogham get github.com/org/database             # добавить зависимость в ogham.toml, скачать и собрать (если плагин)
ogham get github.com/org/database@2.1.0       # добавить конкретную версию
ogham install                                  # установить/собрать все зависимости текущего проекта
ogham install github.com/org/tool@latest      # скачать и собрать бинарник в OGHAM_BIN (глобально)
ogham update                                   # обновить версии в ogham.lock
ogham vendor                                   # скопировать зависимости в vendor/
ogham verify                                   # проверить checksums
```

---

# Plugin System

Плагин — это модуль который определяет аннотации и предоставляет codegen/валидацию.

## Plugin Naming Convention

Бинарники плагинов следуют конвенции `ogham-gen-<name>` (аналогично `protoc-gen-*`):

- `github.com/org/database` → `ogham-gen-database`
- `github.com/org/grpc` → `ogham-gen-grpc`
- `github.com/org/go-pgx` → `ogham-gen-go-pgx`

Имя бинарника выводится автоматически из последнего сегмента пути модуля с префиксом `ogham-gen-`.

**Discovery**: компилятор ищет бинарник в порядке:
1. `$OGHAM_BIN/ogham-gen-<name>@<version>` — конкретная версия
2. `$PATH` — глобально установленные плагины

## Plugin Lifecycle & Distribution

1. **Установка**: `ogham get` или `ogham install` скачивает исходный код в `$OGHAM_CACHE`.
2. **Сборка**: `ogham` автоматически запускает команду `build` из `[plugin]`. Результат: `$OGHAM_BIN/ogham-gen-<name>@<version>`. **build обязателен** для stdio плагинов.
3. **Вызов**: компилятор находит `ogham-gen-<name>` и запускает по протоколу stdio или подключается по gRPC.

## Plugin Protocol

### stdio

Компилятор запускает `ogham-gen-<name>` как процесс. Общение через stdin/stdout.

```
ogham compile → stdin: OghamCompileRequest (protobuf) → [ogham-gen-*] → stdout: OghamCompileResponse (protobuf)
```

В proto mode плагин получает стандартный `google.protobuf.compiler.CodeGeneratorRequest`:

```
ogham compile --proto → stdin: CodeGeneratorRequest (protobuf) → [ogham-gen-*] → stdout: CodeGeneratorResponse (protobuf)
```

### gRPC

Компилятор подключается к запущенному gRPC-сервису плагина.

```
ogham compile → gRPC call: PluginService.Generate(OghamCompileRequest) → OghamCompileResponse
```

gRPC полезен для:
- Тяжёлых плагинов с долгим cold start
- Плагинов как сервисов (shared в CI)
- Watch mode (плагин держит состояние между компиляциями)

## Plugin Manifest

Если модуль является плагином, `ogham.toml` содержит секцию `[plugin]`:

```toml
[package]
name = "github.com/org/database"
version = "2.0.0"
description = "Database codegen plugin for Ogham"

[plugin]
protocol = "stdio"                    # "stdio" | "grpc"
build = "go build -o ogham-gen-database ./cmd"  # ОБЯЗАТЕЛЬНО для stdio

# Для grpc:
# protocol = "grpc"
# build = "go build -o ogham-gen-database ./cmd"  # обязательно — собирает gRPC сервер
# address = "localhost:50051"          # адрес по умолчанию (переопределяется потребителем)

# Что экспортирует плагин
provides = ["annotations", "codegen"] # "annotations" | "codegen" | "validation"

# Целевые языки кодогенерации
targets = ["go", "typescript", "rust"]

[features]
default = ["ogham"]
ogham = []                            # принимает OghamCompileRequest (нативный AST)
proto = []                            # принимает CodeGeneratorRequest (protobuf стандартный)

[plugin.options]
# Опции передаваемые плагину при вызове (настраиваются потребителем)
output_dir = { type = "string", default = "gen/" }
orm = { type = "string", default = "sqlc", enum = ["sqlc", "sqlx", "gorm"] }
```

### Секция [plugin]

| Field | Required | Description |
|-------|----------|-------------|
| `protocol` | yes | Протокол вызова: `stdio` или `grpc` |
| `build` | **yes** | Команда для сборки плагина. Обязательна — плагин всегда предоставляет способ сборки |
| `address` | grpc only | Адрес gRPC сервиса (`host:port`) |
| `provides` | yes | Что предоставляет: `annotations`, `codegen`, `validation` |
| `targets` | codegen only | Целевые языки кодогенерации |

### Feature: `proto`

Плагин может поддерживать два режима входных данных через features:

| Feature | Input | Description |
|---------|-------|-------------|
| `ogham` | `OghamCompileRequest` | Нативный AST ogham — типизированный, полный |
| `proto` | `CodeGeneratorRequest` | Стандартный protobuf — совместимость с protoc экосистемой |

Плагин с `proto` feature умеет работать с `.proto` файлами, что позволяет ему участвовать в proto pipeline наравне со стандартными `protoc-gen-*` плагинами.

### Секция [plugin.options]

Типизированные опции, которые потребитель может переопределить в своём `ogham.toml`:

```toml
# В ogham.toml потребителя
[generate.options."github.com/org/database"]
output_dir = "src/generated/"
orm = "sqlx"
```

## Generation Modes

### Native Mode (default)

Компилятор парсит `.ogham` файлы и отправляет `OghamCompileRequest` плагинам:

```
*.ogham → ogham compiler → OghamCompileRequest → ogham-gen-* plugins → generated code
```

Только `ogham-gen-*` плагины.

### Proto Mode

Компилятор сначала генерирует `.proto` файлы из `.ogham`, затем запускает плагины:

```
*.ogham → ogham compiler → *.proto (с OghamAnnotation options)
                              ↓
                    plugin invocation:
                    ├── ogham-gen-* (с feature proto, получают CodeGeneratorRequest)
                    ├── protoc-gen-* (стандартные protobuf плагины!)
                    └── generated code
```

В proto mode:
- Аннотации сериализуются через `OghamAnnotation { name, google.protobuf.Struct }` (см. `ogham/options.proto`)
- Разрешены стандартные `protoc-gen-*` плагины (`protoc-gen-go`, `protoc-gen-go-grpc`, `protoc-gen-grpc-gateway` и т.д.)
- `protoc-gen-*` плагины ищутся в `$PATH` (стандартный protobuf discovery)
- Все плагины получают одинаковый `CodeGeneratorRequest` — не зависят друг от друга

## Consumer Configuration

Потребитель описывает в своём `ogham.toml` какие плагины использовать и с какими параметрами:

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

# Какие плагины запускать при компиляции и в каком порядке
plugins = [
    # ogham plugins (ogham-gen-*)
    "github.com/org/database",
    "github.com/org/grpc-gen",
    # standard protobuf plugins (protoc-gen-*) — только в proto mode
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

Для разработки плагинов поддерживаются локальные пути. В этом режиме `ogham` следит за изменениями в исходниках плагина.

### Local Path Dependencies

В `ogham.toml` можно указать путь к локальной директории плагина:

```toml
[dependencies]
"my-plugin" = { path = "../plugins/my-plugin" }
```

Компилятор пересоберет плагин при следующем запуске `ogham compile`, если файлы в директории изменились. Бинарник для локального `path` запускается напрямую из места сборки, не загрязняя `OGHAM_BIN`.

### Bootstrapping

Создать заготовку нового плагина в текущей директории:

```bash
ogham init --plugin <name>
```

Это создаст базовый `ogham.toml` с секцией `[plugin]`, команду `build` и структуру файлов. Имя бинарника будет `ogham-gen-<name>`.

## Remote Plugins (gRPC)

Плагины могут работать как удалённые сервисы.

### Serving a Plugin

Если плагин поддерживает протокол `grpc`, его можно запустить как сервер:

```bash
ogham serve --plugin <name> --address :50051
```

Компилятор Ogham умеет подключаться к удалённым плагинам:

```toml
# В ogham.toml потребителя
[dependencies]
"remote-plugin" = { version = "^1.0.0", address = "grpc.prod.internal:50051" }
```

Это позволяет использовать общие плагины в CI/CD или распределённых командах без необходимости установки бинарников на каждую машину.

## Full Example

Проект: Go + PostgreSQL + gRPC с proto mode.

```
myproject/
├── ogham.toml
├── ogham.lock
├── schemas/
│   ├── models.ogham        # package schemas — типы и enum
│   └── api.ogham           # package schemas — сервисы и контракты
├── internal/
│   ├── db/gen/             # ← output от ogham-gen-database
│   ├── api/gen/            # ← output от ogham-gen-grpc
│   └── pb/                 # ← output от protoc-gen-go + protoc-gen-go-grpc
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
    # ogham plugins — читают OghamAnnotation options из .proto
    "github.com/org/database",
    "github.com/org/grpc-gen",
    # standard protobuf plugins — генерируют Go код из .proto
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
ogham compile                         # парсит schemas/, генерирует .proto, вызывает все плагины
ogham compile --plugin=database       # только один плагин
ogham compile --target=go             # только для конкретного target
ogham compile --mode=native           # override: нативный режим вместо proto
```

### Pipeline в proto mode

```
schemas/*.ogham
    ↓ ogham compiler (parse + generate .proto)
schemas/*.proto  (содержат OghamAnnotation options в ogham/options.proto)
    ↓ parallel plugin invocation (все получают одинаковый CodeGeneratorRequest)
    ├── ogham-gen-database    → internal/db/gen/     (читает OghamAnnotation "database::Table" etc.)
    ├── ogham-gen-grpc        → internal/api/gen/    (читает OghamAnnotation "grpc::*")
    ├── protoc-gen-go         → internal/pb/         (генерирует Go structs)
    └── protoc-gen-go-grpc    → internal/pb/         (генерирует gRPC stubs)
```

## Feature Dependency Example

Плагин `ogham-gen-go-pgx` зависит от `database` и требует у него фичу `go`:

```toml
# ogham.toml плагина ogham-gen-go-pgx
[package]
name = "github.com/org/go-pgx"
version = "1.0.0"

[dependencies]
"github.com/org/database" = { version = "^2.0.0", features = ["go", "postgres"] }

[features]
default = ["ogham", "proto"]
ogham = []
proto = []
go = ["github.com/org/database/go"]    # наша "go" фича → включает "go" у database

[plugin]
build = "go build -o ogham-gen-go-pgx ./cmd"
protocol = "stdio"
provides = ["codegen"]
targets = ["go"]
```

Когда потребитель добавляет `go-pgx`, фича `go` на `database` включается автоматически:

```toml
# ogham.toml потребителя
[dependencies]
"github.com/org/go-pgx" = "^1.0.0"
# database автоматически получает features = ["go", "postgres"] через go-pgx
```
