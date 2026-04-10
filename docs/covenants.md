# Covenants

A covenant is a **sum type** (tagged union): a value of a covenant type is always exactly one of its named variants. Think of it like `enum` in Rust or `sealed class` in Kotlin — but with biblical syntax.

Scriptures ask "what does this value **have**?". Covenants ask "what **is** this value?". The answer is always one of its variants.

---

## Declaring a covenant

Variants can be **unit** (no data) or **data-carrying** (fields indented below the name):

```holy
covenant Direction
    North
    South
    East
    West
```

```holy
covenant Shape
    Circle
        radius of fractional
    Rectangle
        width  of fractional
        height of fractional
    Point           -- unit variant (no fields)
```

- At least one variant is required.
- A variant with an indented block is a **data-carrying variant**.
- A variant without a block is a **unit variant**.

---

## Creating a value

### Unit variant

```holy
let there d of Direction be North of Direction
```

The `of CovenantName` suffix is required — the interpreter needs to know which covenant the variant belongs to.

### Data-carrying variant — `manifest`

```holy
let there s of Shape be manifest Circle    of Shape praying 5.0
let there r of Shape be manifest Rectangle of Shape praying 3.0, 4.0
let there q of Shape be manifest Rectangle of Shape praying 3.0 and 4.0
```

Arguments follow the **field declaration order**. The final separator may be `and`.

---

## Pattern matching — `discern`

`discern` inspects the value and executes the branch matching the current variant:

```holy
discern d
    as North
        hail proclaim praying "going up"
    as South
        hail proclaim praying "going down"
    otherwise
        hail proclaim praying "going sideways"
```

- At least one `as` branch is required.
- `otherwise` (optional) catches any variant not listed.
- If no branch matches and there is no `otherwise`, an `InvalidDiscern` sin is thrown at runtime.

### Binding fields — `bearing`

Use `bearing name1, name2, …` after the variant name to bind its fields to local variables:

```holy
discern s
    as Circle bearing r
        hail proclaim praying "circle, radius " plus hail word_of praying r
    as Rectangle bearing w and h
        let there area of fractional be w times h
        hail proclaim praying "rectangle, area " plus hail word_of praying area
    as Point
        hail proclaim praying "just a point"
```

- Bindings are **positional** (same order as the declared fields).
- You can bind fewer names than there are fields — extras are ignored.
- Unit variants never use `bearing`.

---

## Generic covenants

Covenants can declare type parameters with `of`:

```holy
covenant Option of T
    Some
        value of T
    None

covenant Either of L and R
    Left
        val of L
    Right
        val of R
```

When instantiating, pass the types explicitly:

```holy
let there o of Option of atom be manifest Some of Option of atom praying 42
let there n of Option of atom be None of Option of atom
let there e of Either of atom and word be manifest Left of Either of atom and word praying 7
```

---

## Built-in covenants

Two covenants are pre-loaded in every program. They are generic and have runtime type checking.

---

### `grace of T` — optional value

Equivalent to `Option` / `Maybe` in other languages. Represents "may or may not have a value".

| Variant   | Fields | Meaning            |
|-----------|--------|--------------------|
| `granted` | `T`    | a value is present |
| `absent`  | —      | no value (unit variant) |

```holy
-- creating
let there g of grace of atom be manifest granted of grace of atom praying 42
let there n of grace of atom be absent of grace of atom

-- default when declared without an initialiser
let there be x of grace of word    -- x = absent
```

```holy
-- using
discern g
    as granted bearing value
        hail proclaim praying hail word_of praying value   -- "42"
    as absent
        hail proclaim praying "nothing here"
```

`manifest granted of grace of atom praying "text"` throws `TypeError` — the inner value must be of type `atom`.

---

### `verdict of T and E` — fallible result

Equivalent to `Result` in other languages. Represents "an operation that may succeed or fail".

| Variant      | Fields | Meaning               |
|--------------|--------|-----------------------|
| `righteous`  | `T`    | operation succeeded   |
| `condemned`  | `E`    | operation failed      |

```holy
-- creating
let there r of verdict of atom and word be manifest righteous of verdict of atom and word praying 99
let there e of verdict of atom and word be manifest condemned of verdict of atom and word praying "invalid input"
```

```holy
-- using
discern r
    as righteous bearing value
        hail proclaim praying hail word_of praying value    -- "99"
    as condemned bearing reason
        hail proclaim praying reason
```

#### Nested generic types

When `T` or `E` is itself generic, use `thus` to close the inner type before the outer separator:

```holy
-- verdict<Stack<atom>, word>
let there result of verdict of Stack of atom thus and word be hail pop of atom praying s
```

See [Generics — `thus`](generics.md#thus--disambiguation) for the full rule.

---

## Full example

```holy
covenant Direction
    North
    South
    East
    West

salm move upon Direction receiving steps of atom reveals word
    discern its
        as North
            reveal "moved up " plus hail word_of praying steps
        as South
            reveal "moved down " plus hail word_of praying steps
        as East
            reveal "moved east " plus hail word_of praying steps
        as West
            reveal "moved west " plus hail word_of praying steps

let there d of Direction be North of Direction
hail proclaim praying hail move upon d praying 3    -- "moved up 3"

d become East of Direction
hail proclaim praying hail move upon d praying 5    -- "moved east 5"

amen
```
