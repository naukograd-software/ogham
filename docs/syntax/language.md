# Ogham Language Syntax Reference

## Package System

Packages работают как в Go. Каждый файл начинается с объявления пакета. Файлы одного пакета видят типы друг друга напрямую.

```
package <name>;
import <path>;
import <path> as <alias>;
```

**Видимость**: имена с заглавной буквы экспортируются, с маленькой — только внутри пакета.

## Namespace Resolution

- **Типы** разрешаются через `.` — `uuid.UUID`, `std.Timestamp`
- **Аннотации** вызываются через `::` — `@database::Table(...)`

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
| `[N]T` | Fixed-size array of N elements |
| `map<K, V>` | Map (K must be comparable) |

## Type

Структура с нумерованными полями. Поддерживает wire-совместимость через явные номера полей.

```
type Name {
    <type> <field_name> = <field_number>;
}
```

**Type alias** — compile-time синоним:

```
type Name = OtherType;
```

**Generic type** — compile-time monomorphization (не runtime generic):

```
type Name<T> {
    []T data = 1;
}
```

**Вложенные типы** — всегда reference (как protobuf message), не value. Циклические зависимости разрешены.

## Shape

Набор полей без нумерации. Используется как миксин для композиции в type.

```
shape Name {
    <type> <field_name>;
}
```

**Композиция** — shape может включать другие shapes:

```
shape Combined {
    ShapeA;
    ShapeB, ShapeC;
}
```

**Инъекция в type** — shape встраивается с явным range номеров полей:

```
type Model {
    MyShape(1..4)
    <type> next_field = 5;
}
```

Компилятор проверяет что shape помещается в range `1..4`. Если shape вырастет за пределы range — ошибка компиляции. Range указывается с запасом если планируется рост shape.

## Enum

```
enum Name {
    Value1 = 1;
    Value2 = 2;
}
```

`Unspecified = 0` добавляется неявно.

**Удаление значений**: `@removed(fallback=<non-removed-value>)`. Fallback обязан указывать на не-removed значение. Цепочки fallback запрещены.

## Oneof

Определяется только внутри type. Поля нумеруются в пространстве родительского type.

```
type Model {
    oneof field_name {
        TypeA variant_a = 2;
        TypeB variant_b = 3;
    }
}
```

Допускается несколько полей одного типа с разными номерами.

## Service & Contract

Service — группа RPC-контрактов. Contract определяет вход и выход.

```
service Name {
    contract MethodName(InputType) -> OutputType;
}
```

- `void` — отсутствие входа или выхода
- Inline type `{ fields }` — компилятор генерирует имя `<ContractName>Input` / `<ContractName>Output`
- Generic в возврате (`Paginated<T>`) — compile-time monomorphization

## Keywords: Pick & Omit

Встроенные keyword языка (не из библиотек).

**Pick** — создаёт тип из подмножества полей:

```
type Sub = Pick<Original, field1, field2>;
```

**Omit** — создаёт тип исключая поля:

```
type Without = Omit<Original, field1, field2>;
type Without2 = Omit<Original, ShapeName>;
```

При Omit по shape — сопоставление по паре имя+тип. Компилятор выдаёт warning если ничего не исключилось.

## Annotations

### Definition

Аннотация определяется в библиотеке с явным указанием целей и схемой параметров.

```
annotation Name for <target1>|<target2> {
    <type> <param_name>;
    <type>? <param_name> = <default>;
}
```

**Targets**: `shape`, `type`, `field`, `oneof`, `oneof_field`, `enum`, `enum_value`, `service`, `contract`.

Поле опционально если имеет `?` или значение по умолчанию.

### Call

```
@<library>::<AnnotationName>(<param>=<value>, ...)
```

### Built-in Annotations

| Annotation | Description | Proto mapping |
|------------|-------------|---------------|
| `@default(<value>)` | Значение по умолчанию. Magic keywords: `now`, `(u)int*.<min,max>` | Custom option `ogham.default` |
| `@cast(<type>)` | Безопасное приведение типа | Custom option `ogham.cast` |
| `@removed(fallback=<value>)` | Пометить enum value как логически удалённое | Custom options `ogham.removed` + `ogham.fallback` (значение остаётся в proto enum — proto enum не удаляет values) |
| `@reserved(<number>)` | Зарезервировать номер поля | `reserved <number>;` |

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
| `shape` | `google.protobuf.MessageOptions` (проваливается на message, в который shape инжектирован) |

## Semicolons

Точка с запятой обязательна после всех объявлений: полей, type alias, enum values, contracts.

## Protobuf Compatibility

Ogham полностью совместим с protobuf: из любой `.ogham` схемы можно сгенерировать корректный `.proto` файл.

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
| `[N]T` | `repeated T` (размер — compile-time constraint) |
| `map<K, V>` | `map<K, V>` (ключи всегда comparable, конвертируются к proto-типу) |

### Structural Mapping

| Ogham | Proto |
|-------|-------|
| `type` | `message` |
| `type Alias = T` | Раскрывается в целевой тип |
| `type Generic<T>` | Monomorphization → конкретные `message` |
| `enum` | `enum` (все значения сохраняются, `@removed` → option) |
| `shape` | Раскрывается в поля `message` |
| `Pick<T, ...>` / `Omit<T, ...>` | Новый `message` с подмножеством полей |
| `oneof` | `oneof` |
| `service` | `service` |
| `contract` | `rpc` |
| `void` | `google.protobuf.Empty` |

### Annotations → OghamAnnotation

Все аннотации сериализуются через единый extension `OghamAnnotation` с `google.protobuf.Struct`:

```protobuf
// ogham/options.proto — часть ogham std
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

Не нужна нумерация в annotation declarations — компилятор валидирует типы, Struct используется как транспорт. Пример:

```
// Ogham source:
@database::Table(table_name="users")
type User { ... }

// Generated .proto:
message User {
    option (ogham) = { name: "database::Table", params: { fields { key: "table_name" value { string_value: "users" } } } };
}
```
