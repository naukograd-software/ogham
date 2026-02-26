# Package Management

## Module System

Работает как в Go. Модуль — корневая единица, идентифицируемая URL-путём. Пакет — директория внутри модуля.

```
myproject/
├── ogham.toml          # манифест модуля
├── ogham.lock          # lock-file (генерируется автоматически)
├── models/
│   ├── user.ogham      # package models
│   └── order.ogham     # package models
├── api/
│   └── contracts.ogham # package api
└── vendor/             # опционально, как go vendor
```

Файлы в одной директории — один пакет. Они видят типы друг друга напрямую без import.

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
import mydb = github.com/org/database;
import otherdb = github.com/other/database;
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
"github.com/org/database" = { version = "^2.0.0", features = ["postgres"] }

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

`default` — features включённые по умолчанию. Потребитель может переопределить:

```toml
[dependencies]
"github.com/org/project" = { version = "^1.0.0", default-features = false, features = ["rest-api"] }
```

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
```

## CLI Commands

```
ogham get github.com/org/database             # добавить зависимость
ogham get github.com/org/database@2.1.0       # конкретная версия
ogham update                                   # обновить зависимости в рамках semver
ogham vendor                                   # скопировать зависимости в vendor/
ogham verify                                   # проверить checksums
```

---

# Plugin System

Плагин (библиотека) — это модуль который определяет аннотации и предоставляет codegen/валидацию. Компилятор вызывает плагин, передаёт AST со всеми аннотациями, плагин возвращает сгенерированный код.

## Plugin Protocol

Два способа вызова:

### stdio

Компилятор запускает бинарник плагина как процесс. Общение через stdin/stdout (как protoc-gen-*).

```
ogham compile → stdin: CompileRequest (protobuf) → [plugin binary] → stdout: CompileResponse (protobuf)
```

### gRPC

Компилятор подключается к запущенному gRPC-сервису плагина.

```
ogham compile → gRPC call: PluginService.Generate(CompileRequest) → CompileResponse
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
binary = "ogham-plugin-database"      # имя бинарника для stdio

# Для grpc:
# protocol = "grpc"
# address = "localhost:50051"          # адрес по умолчанию (переопределяется через CLI)

# Что экспортирует плагин
provides = ["annotations", "codegen"] # "annotations" | "codegen" | "validation"

# Целевые языки кодогенерации
targets = ["go", "typescript", "rust"]

[plugin.options]
# Опции передаваемые плагину при вызове (настраиваются потребителем)
output_dir = { type = "string", default = "gen/" }
orm = { type = "string", default = "sqlc", enum = ["sqlc", "sqlx", "gorm"] }
```

### Секция [plugin]

| Field | Required | Description |
|-------|----------|-------------|
| `protocol` | yes | Протокол вызова: `stdio` или `grpc` |
| `binary` | stdio only | Имя бинарника в PATH или относительный путь |
| `address` | grpc only | Адрес gRPC сервиса (`host:port`) |
| `provides` | yes | Что предоставляет: `annotations`, `codegen`, `validation` |
| `targets` | codegen only | Целевые языки кодогенерации |

### Секция [plugin.options]

Типизированные опции, которые потребитель может переопределить в своём `ogham.toml`:

```toml
# В ogham.toml потребителя
[dependencies.options."github.com/org/database"]
output_dir = "src/generated/"
orm = "sqlx"
```

## Plugin Lifecycle

### stdio

1. Компилятор парсит `.ogham` файлы в AST
2. Для каждого плагина (зависимости с `[plugin]`):
   - Запускает `binary` как процесс
   - Пишет `CompileRequest` в stdin (protobuf-encoded)
   - Читает `CompileResponse` из stdout
   - Процесс завершается
3. Компилятор записывает сгенерированные файлы

### gRPC

1. Компилятор парсит `.ogham` файлы в AST
2. Для каждого gRPC плагина:
   - Подключается к `address`
   - Вызывает `PluginService.Generate(CompileRequest)`
   - Получает `CompileResponse`
   - Соединение переиспользуется в watch mode
3. Компилятор записывает сгенерированные файлы

## Consumer Configuration

Потребитель описывает в своём `ogham.toml` какие плагины использовать и с какими параметрами:

```toml
[package]
name = "github.com/myteam/myproject"
version = "0.1.0"

[dependencies]
"github.com/ogham/std" = "^1.0.0"
"github.com/ogham/uuid" = "^1.0.0"
"github.com/org/database" = { version = "^2.0.0", features = ["postgres"] }
"github.com/org/grpc-gen" = "^1.0.0"

[generate]
# Какие плагины запускать при компиляции и в каком порядке
plugins = [
    "github.com/org/database",
    "github.com/org/grpc-gen",
]

[generate.options."github.com/org/database"]
output_dir = "internal/db/gen/"
orm = "sqlx"

[generate.options."github.com/org/grpc-gen"]
output_dir = "internal/api/gen/"
target = "go"
```

## Full Example

Структура проекта с двумя плагинами:

```
myproject/
├── ogham.toml
├── ogham.lock
├── schemas/
│   ├── models.ogham        # package schemas — типы и enum
│   └── api.ogham           # package schemas — сервисы и контракты
├── internal/
│   ├── db/gen/             # ← output от database плагина
│   └── api/gen/            # ← output от grpc-gen плагина
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
"github.com/org/database" = { version = "^2.0.0", features = ["postgres"] }
"github.com/org/grpc-gen" = { version = "^1.0.0", features = ["grpc-gateway"] }

[features]
default = ["postgres", "grpc"]
postgres = []
grpc = []

[generate]
plugins = [
    "github.com/org/database",
    "github.com/org/grpc-gen",
]

[generate.options."github.com/org/database"]
output_dir = "internal/db/gen/"
orm = "sqlx"

[generate.options."github.com/org/grpc-gen"]
output_dir = "internal/api/gen/"
target = "go"
```

```
ogham compile                         # парсит schemas/, вызывает плагины, пишет в gen/
ogham compile --plugin=database       # только один плагин
ogham compile --target=go             # только для конкретного target
```
