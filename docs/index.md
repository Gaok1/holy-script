# Holy Script — Getting Started

Holy is an interpreted, strongly typed programming language with archaic/biblical syntax, implemented in Rust.

---

## Program structure

Every `.holy` file follows the same layout:

```
[testaments]       ← imports (optional)
[declarations]     ← scripture, sin, covenant, salm (optional)
[statements]       ← executable code
amen               ← required, ends the file
```

`amen` is mandatory and must be the last thing in the file.

---

## Hello, world

```holy
hail proclaim praying "Hail, world!"

amen
```

Run with:

```bash
holy hello.holy
```

---

## A complete program

```holy
-- Data structure
scripture Person
    name of word
    age  of atom

-- Function (salm)
salm greet upon Person reveals void
    hail proclaim praying "Hail, " plus name from its plus "!"

-- Executable code
let there p of Person be manifest Person praying "Gabriel", 30
hail greet upon p

amen
```

---

## Core concepts

| Concept   | Keyword      | Equivalent in other languages    |
|-----------|-------------|----------------------------------|
| Variable  | `let there`  | `let`, `var`, `int x`            |
| Function  | `salm`       | `func`, `def`, `fn`              |
| Struct    | `scripture`  | `struct`, `class` (data only)    |
| Enum      | `covenant`   | `enum`, `sealed class`           |
| Exception | `sin`        | `exception`, `error`             |
| Loop      | `litany for` | `while`                          |
| If        | `whether`    | `if`/`else if`/`else`            |
| Throw     | `transgress` | `throw`, `raise`                 |
| Try/Catch | `confess`    | `try`/`catch`/`finally`          |
| Return    | `reveal`     | `return`                         |
| Print     | `hail proclaim praying` | `print`, `console.log` |

---

## Primitive types

| Type         | Meaning            | Example                   |
|--------------|--------------------|---------------------------|
| `atom`       | integer (i64)      | `42`, `-7`                |
| `fractional` | decimal (f64)      | `3.14`, `-0.5`            |
| `word`       | text (UTF-8)       | `"hello"`                 |
| `dogma`      | boolean            | `blessed`, `forsaken`     |
| `void`       | no value           | —                         |
| `legion of T`| typed collection   | `hail legion praying 1, 2 and 3` |

`blessed` = true · `forsaken` = false

---

## Documentation by topic

| Topic | Description |
|-------|-------------|
| [Types & Variables](types.md) | Primitives, literals, variables, operators, grouping |
| [Collections](collections.md) | `legion of T`, creation and methods |
| [Salms](salms.md) | Functions, parameters, return values, built-in salms |
| [Control Flow](control-flow.md) | `whether`, `litany for`, `forsake`, `ascend` |
| [Scriptures](scriptures.md) | Structs, field access, method salms |
| [Covenants](covenants.md) | Sum types, pattern matching with `discern` |
| [Sins](sins.md) | Exceptions, `transgress`, `confess`/`answer for`/`absolve` |
| [Generics](generics.md) | Type parameters, `thus` for disambiguation |
| [Nesting](nesting.md) | Full reference for `thus` and `after` |
| [Modules](modules.md) | `testament`, selective imports with `revealing` |
