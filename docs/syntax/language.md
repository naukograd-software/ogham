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

| Annotation | Description |
|------------|-------------|
| `@default(<value>)` | Значение по умолчанию. Magic keywords: `now`, `(u)int*.<min,max>` |
| `@cast(<type>)` | Безопасное приведение типа |
| `@removed(fallback=<value>)` | Пометить enum value как удалённое |
| `@reserved(<number>)` | Зарезервировать номер поля |

## Semicolons

Точка с запятой обязательна после всех объявлений: полей, type alias, enum values, contracts.
