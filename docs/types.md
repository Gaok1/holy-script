# Types & Variables

Holy is strongly typed: every value has a declared type and the runtime rejects values that do not match. There is no implicit conversion.

---

## Primitive types

| Type         | Meaning             | Default       | Literal examples        |
|--------------|---------------------|---------------|-------------------------|
| `atom`       | integer (i64)       | `0`           | `42`, `-7`, `0`         |
| `fractional` | decimal (f64)       | `0.0`         | `3.14`, `-0.5`          |
| `word`       | text (UTF-8)        | `""`          | `"hello"`, `""`         |
| `dogma`      | boolean             | `forsaken`    | `blessed`, `forsaken`   |
| `void`       | no value            | —             | —                       |
| `legion of T`| typed collection    | empty         | see [Collections](collections.md) |

`blessed` = true · `forsaken` = false

---

## Variables

Holy offers four ways to declare variables, with increasing flexibility.

### Typed declaration with initial value

The type is explicit and the initial value is checked against it immediately:

```holy
let there x of atom be 42
let there greeting of word be "Hail, world!"
let there active of dogma be blessed
let there ratio of fractional be 3.14
```

### Typed declaration without initial value

Initialises with the type's default value (`0`, `""`, `forsaken`, etc.):

```holy
let there be x of atom          -- x = 0
let there be msg of word        -- msg = ""
let there be flag of dogma      -- flag = forsaken
```

### Inferred type — `let there x be expr`

The type is derived from the expression's value and locked permanently:

```holy
let there n be 42               -- n: atom (locked)
let there greeting be "Hail!"   -- greeting: word (locked)
let there xs be hail legion praying 1, 2 and 3   -- xs: legion of atom (locked)
```

Once the type is inferred, `become` only accepts values of that same type:

```holy
n become 99         -- ok: atom
n become "oops"     -- TypeError: demands atom, received word
```

For `legion`, the inner type is inferred from the first element. A literally empty legion has no inner type (generic `legion`), which accepts any element — prefer declaring the type explicitly in that case.

### Untyped declaration — `let there be x`

The variable exists but has no type. The type is inferred and locked on the **first** assignment via `become`:

```holy
let there be result

whether condition
    result become 42
otherwise
    result become 0

-- result is now atom, locked by whichever branch ran first
result become 99    -- ok
result become "no"  -- TypeError
```

The type is determined at runtime by whichever branch executes first — not by the compiler.

### Reassignment

```holy
x become x plus 1
greeting become "Farewell"
```

The new value must match the variable's already-locked type.

---

## Operators

### Arithmetic

| Expression         | Operation      | Works with                               |
|--------------------|----------------|------------------------------------------|
| `a plus b`         | addition       | `atom`, `fractional`, `word` (concat)    |
| `a minus b`        | subtraction    | `atom`, `fractional`                     |
| `a times b`        | multiplication | `atom`, `fractional`                     |
| `a over b`         | division       | `atom` (integer), `fractional`           |
| `a remainder b`    | modulo         | `atom`                                   |
| `negate a`         | unary minus    | `atom`, `fractional`                     |

```holy
let there x of atom be 10 remainder 3       -- 1
let there y of atom be 7 over 2             -- 3  (integer division!)
let there z of fractional be 7.0 over 2     -- 3.5
let there s of word be "Holy" plus " Lang"  -- "Holy Lang"
```

Mixing `atom` and `fractional` promotes both to `fractional`.

### Comparison

| Expression             | Meaning          |
|------------------------|------------------|
| `a is b`               | equal            |
| `a is not b`           | not equal        |
| `a greater than b`     | greater than     |
| `a lesser than b`      | less than        |
| `a no greater than b`  | less or equal    |
| `a no lesser than b`   | greater or equal |

`is` / `is not` work on any type. Ordered comparisons work on `atom` and `fractional`.

---

## Operator precedence

From lowest to highest:

| Level | Operators                          |
|-------|------------------------------------|
| 1     | comparisons (`is`, `greater`, …)   |
| 2     | `plus`, `minus`                    |
| 3     | `times`, `over`, `remainder`       |
| 4     | `negate` (unary)                   |
| 5     | atoms (literals, variables, calls) |

```holy
-- times has higher precedence than plus:
-- 2 plus 3 times 4  →  2 + (3 * 4) = 14
let there x of atom be 2 plus 3 times 4
```

---

## Expression grouping — `after`

`after` deepens the parser into full expression parsing, equivalent to opening a parenthesis. `thus` is **optional**: it is only needed when the outer expression must continue after the group.

```holy
after 2 plus 3 thus times 4   -- (2 + 3) * 4 = 20  (thus closes the group early)
5 plus after 10 minus 3       -- 5 + (10 - 3) = 12  (no thus needed)
a times after a plus b        -- a * (a + b)         (no thus needed)
```

See more cases in [Nesting](nesting.md).

---

## Type conversion

There is no implicit coercion. Use the built-in salms:

```holy
-- word → atom
let there n of atom be hail atom_of praying "42"

-- anything → word
let there s of word be hail word_of praying 99
let there b of word be hail word_of praying blessed
```

---

## Built-in `word` methods

| Method | Returns | Description |
|--------|---------|-------------|
| `hail length upon s` | `atom` | number of characters |
| `hail is_empty upon s` | `dogma` | whether the text is empty |
| `hail at upon s praying i` | `word` | character at index `i` (zero-based) |
| `hail slice upon s praying start and end` | `word` | substring `[start, end)` |
| `hail contains upon s praying sub` | `dogma` | whether `sub` is contained in `s` |
| `hail starts_with upon s praying prefix` | `dogma` | whether `s` starts with `prefix` |
| `hail ends_with upon s praying suffix` | `dogma` | whether `s` ends with `suffix` |
| `hail index_of upon s praying sub` | `grace of atom` | position of `sub` or `absent` |
| `hail to_upper upon s` | `word` | uppercase |
| `hail to_lower upon s` | `word` | lowercase |
| `hail trim upon s` | `word` | strips leading and trailing whitespace |
| `hail replace upon s praying old and new` | `word` | replaces all occurrences |
| `hail split upon s praying sep` | `legion of word` | splits by separator `sep` |

```holy
let there s of word be "Hello, World!"
hail proclaim praying hail to_upper upon s                    -- "HELLO, WORLD!"
hail proclaim praying hail contains upon s praying "World"    -- blessed
hail proclaim praying hail slice upon s praying 0 and 5      -- "Hello"
hail proclaim praying hail replace upon s praying "World" and "Holy"  -- "Hello, Holy!"

let there parts of legion of word be hail split upon "a,b,c" praying ","
-- parts = ["a", "b", "c"]

let there idx of grace of atom be hail index_of upon s praying "World"
discern idx
    as granted bearing i
        hail proclaim praying hail word_of praying i    -- 7
    as absent
        hail proclaim praying "not found"
```

`at` and `slice` throw `IndexOutOfBounds` for out-of-range indices.

---

## Built-in `legion of T` methods

See [Collections](collections.md) for the full reference.
