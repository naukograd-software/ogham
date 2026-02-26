# Package Management

## Environment and Storage

Ogham использует глобальное хранилище для зависимостей и бинарников, что позволяет избежать дублирования кода в каждом проекте.

- **`OGHAM_HOME`**: Корневая директория Ogham (по умолчанию `~/.ogham`).
- **`OGHAM_BIN`**: Директория для скомпилированных плагинов (по умолчанию `$OGHAM_HOME/bin`).
- **`OGHAM_CACHE`**: Директория для скачанных исходных кодов пакетов (по умолчанию `$OGHAM_HOME/pkg/mod`).

### Directory Structure

```
$OGHAM_HOME/
├── bin/                # Скомпилированные бинарники плагинов
│   ├── database@v2.0.0
│   └── grpc-gen@v1.0.3
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

Плагин (библиотека) — это модуль который определяет аннотации и предоставляет codegen/валидацию. 

## Plugin Lifecycle & Distribution

Процесс использования плагина полностью автоматизирован:

1.  **Установка**: При запуске `ogham get` (для нового плагина) или `ogham install` (для существующего проекта), исходный код плагина скачивается в `$OGHAM_CACHE`.
2.  **Прозрачная сборка**: Если в `ogham.toml` плагина указано поле `build`, `ogham` автоматически запускает сборку. Результирующий бинарник сохраняется в `$OGHAM_BIN/<name>@<version>`.
3.  **Вызов**: При компиляции проекта `ogham` ищет нужную версию бинарника в `$OGHAM_BIN`. 

Команда `ogham install <path>` позволяет установить плагин или утилиту в `$OGHAM_BIN` вне контекста конкретного проекта.

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
| `binary` | stdio only | Путь к бинарнику относительно корня плагина (после сборки будет скопирован в `OGHAM_BIN`) |
| `build` | no | Команда для сборки плагина из исходников (запускается автоматически при установке или изменении `path`) |
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

## Local Development

Для разработки плагинов поддерживаются локальные пути. В этом режиме `ogham` следит за изменениями в исходниках плагина.

### Local Path Dependencies

В `ogham.toml` можно указать путь к локальной директории плагина:

```toml
[dependencies]
"my-plugin" = { path = "../plugins/my-plugin" }
```

Если у такого плагина есть поле `build`, компилятор пересоберет его при следующем запуске `ogham compile`, если файлы в директории изменились. Бинарник для локального `path` может запускаться напрямую из места сборки, не загрязняя `OGHAM_BIN`.

### Bootstrapping

Создать заготовку нового плагина в текущей директории:

```bash
ogham init --plugin <name>
```

Это создаст базовый `ogham.toml` с секцией `[plugin]` и структуру файлов.

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
