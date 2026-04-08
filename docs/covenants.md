# Covenants

Covenants are sum types (tagged unions). A value of a covenant type is always exactly one of its named variants. Variants can be unit (carry no data) or carry named fields.

---

## Declaration

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
- A variant with an indented field block is a **data variant**.
- A variant with no indented block is a **unit variant**.

---

## Instantiation

### Unit variant

Reference the variant name directly, followed by `of CovenantName`:

```holy
let there d of Direction be North of Direction
```

> The `of CovenantName` suffix is required for type clarity. The interpreter needs to know which covenant the variant belongs to.

### Data variant — `manifest`

Use `manifest` to supply field values:

```holy
let there s of Shape be manifest Circle of Shape praying 5.0
let there r of Shape be manifest Rectangle of Shape praying 3.0, 4.0
let there q of Shape be manifest Rectangle of Shape praying 3.0 and 4.0
```

Arguments are in **field declaration order**. As with other Holy lists, the final separator may be `and`.

---

## Pattern matching — `discern`

`discern` matches a value against its covenant variants. At least one `as` branch is required. An optional `otherwise` handles any unmatched variant.

```holy
discern d
    as North
        hail proclaim praying "going up"
    as South
        hail proclaim praying "going down"
    otherwise
        hail proclaim praying "going sideways"
```

### Binding fields — `bearing`

Add `bearing name1, name2, …` after the variant name to bind its fields positionally to local variables. The final separator may also be `and`:

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

- Bindings are positional (same order as declared fields).
- You may bind fewer names than there are fields (extra fields are ignored).
- A unit variant never uses `bearing`.

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

When instantiating, supply type args explicitly. The final type separator may be `and`:

```holy
let there o of Option of atom be manifest Some of Option of atom praying 42
let there n of Option of atom be None of Option of atom
let there e of Either of atom and word be manifest Left of Either of atom and word praying 7
```

See [Generics](generics.md) for generic-type rules and [Disambiguation with `thus` and `after`](nesting.md) for the general ambiguity model.

---

## Built-in covenants

Two covenants are pre-loaded into every program. They are fully generic and have first-class runtime type checking.

---

### `grace of T` — optional value

Equivalent to `Option` / `Maybe` in other languages.

| Variant   | Fields | Meaning |
|-----------|--------|---------|
| `granted` | `T`    | a value was given |
| `absent`  | —      | nothing is present (unit variant) |

```holy
-- creating
let there g of grace of atom be manifest granted of grace of atom praying 42
let there n of grace of atom be absent of grace of atom

-- default value when declared without initialiser
let there be x of grace of word    -- x = absent
```

```holy
-- matching
discern g
    as granted bearing value
        hail proclaim praying hail word_of praying value
    as absent
        hail proclaim praying "nothing here"
```

`manifest granted of grace of atom praying "x"` raises `TypeError` — the inner value must match the declared `T`.

---

### `verdict of T and E` — fallible result

Equivalent to `Result` in other languages.

| Variant     | Fields | Meaning |
|-------------|--------|---------|
| `righteous` | `T`    | operation succeeded |
| `condemned` | `E`    | operation failed    |

```holy
-- creating
let there r of verdict of atom and word be manifest righteous of verdict of atom and word praying 99
let there e of verdict of atom and word be manifest condemned of verdict of atom and word praying "bad input"
```

```holy
-- matching
discern r
    as righteous bearing value
        hail proclaim praying hail word_of praying value
    as condemned bearing reason
        hail proclaim praying reason
```

#### Nested generic types

When `T` or `E` is itself generic, use `thus` to close the inner type before the outer separator:

```holy
-- verdict<Stack<atom>, word>
let there result of verdict of Stack of atom thus and word be hail pop of atom praying s

discern result
    as righteous bearing newStack
        hail proclaim praying "popped"
    as condemned bearing reason
        hail proclaim praying reason
```

See [Generics — `thus`](generics.md#thus--disambiguation) and [Disambiguation with `thus` and `after`](nesting.md#4-disambiguating-nested-generic-types) for the full rule.

---

## `discern` and `otherwise`

`otherwise` at the end of a `discern` block acts as a catch-all:

```holy
discern status
    as Active
        hail proclaim praying "online"
    otherwise
        hail proclaim praying "offline"
```

If no branch matches and there is no `otherwise`, a runtime sin `InvalidDiscern` is raised.
