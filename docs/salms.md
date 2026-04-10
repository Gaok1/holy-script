# Salms

Salms are functions. Each salm explicitly declares its parameters and return type, and returns a value with `reveal`.

---

## Basic declaration

```holy
salm add receiving a of atom, b of atom reveals atom
    reveal a plus b

salm greet reveals void
    hail proclaim praying "Hail!"
```

- `receiving param_list` — optional; omit if there are no parameters.
- `reveals type` — required; use `void` when the salm produces no value.
- The body is an indented block with at least one statement.
- In Holy lists, the final separator may be `and`: `a and b`, `a, b and c`.

---

## Calling a salm — `hail`

```holy
-- no arguments
hail greet

-- with arguments
let there result of atom be hail add praying 3, 5
let there result of atom be hail add praying 3 and 5   -- equivalent

-- as a statement (discards the return value)
hail proclaim praying "done"
```

### Nested calls

A salm can be an argument to another:

```holy
-- proclaim(word_of(42))
hail proclaim praying hail word_of praying 42
```

When the inner call is **not the last argument** of the outer one, use `thus` to close it:

```holy
-- add(double(3), 1) = 7
let there y of atom be hail add praying hail double praying 3 thus and 1
```

Without `thus`, `and 1` would be consumed as a second argument to `double`. See [Nesting](nesting.md) for all cases.

---

## `reveal` — return

`reveal expr` returns a value and exits the salm immediately:

```holy
salm max receiving a of atom, b of atom reveals atom
    whether a greater than b
        reveal a
    reveal b
```

`reveal` may appear anywhere in the body — inside `whether`, `discern`, `litany`, etc.

A `void` salm may omit `reveal` entirely (it exits when the block ends).

---

## Parameters

Each parameter has an explicit name and type. The final separator may be `and`:

```holy
salm describe receiving name of word, age of atom and score of fractional reveals word
    reveal name plus " (" plus hail word_of praying age plus ")"
```

There is no variadic syntax. If you need a variable number of elements, pass a `legion`.

---

## Generic salms

Declare type parameters with `of` after the salm name:

```holy
salm identity of T receiving val of T reveals T
    reveal val

salm wrap of T receiving val of T reveals grace of T
    reveal manifest granted of grace of T praying val
```

Pass types explicitly at the call site:

```holy
let there g of grace of atom be hail wrap of atom praying 42
let there w of grace of word be hail wrap of word praying "hello"
```

Type parameters are erased at runtime — the interpreter accepts any value for an abstract type without checking. Concrete types (`atom`, `word`, registered scriptures/covenants) continue to be verified.

---

## Method salms

Declared with `upon TargetType`. See [Scriptures — Method salms](scriptures.md#method-salms).

```holy
salm area upon Rectangle reveals fractional
    reveal width from its times height from its

hail area upon rect
```

---

## Built-in salms

Available in every program without declaration:

### I/O

| Salm | Returns | Description |
|------|---------|-------------|
| `proclaim` | `void` | Prints with newline |
| `herald` | `void` | Prints without newline |
| `inquire` | `word` | Reads one line from stdin |
| `read_file` | `verdict of word and word` | Reads a file; righteous(contents) or condemned(error) |
| `write_file` | `verdict of dogma and word` | Writes a file; righteous(blessed) or condemned(error) |
| `args` | `legion of word` | Command-line arguments passed to the script |
| `exit` | `void` | Exits the program with an exit code |

### Type conversion

| Salm | Returns | Description |
|------|---------|-------------|
| `atom_of` | `atom` | Converts text to integer (0 if invalid) |
| `parse_atom` | `verdict of atom and word` | Converts text to integer with result |
| `fractional_of` | `fractional` | Converts to decimal |
| `word_of` | `word` | Converts any value to text |

### Math

| Salm | Returns | Description |
|------|---------|-------------|
| `abs` | `atom`/`fractional` | Absolute value |
| `floor` | `atom` | Rounds down to the nearest integer |
| `ceil` | `atom` | Rounds up to the nearest integer |
| `round` | `atom` | Rounds to the nearest integer |
| `min` | `atom`/`fractional` | Smaller of two values |
| `max` | `atom`/`fractional` | Larger of two values |
| `pow` | `atom`/`fractional` | Exponentiation (`pow(base, exp)`) |
| `sqrt` | `fractional` | Square root |

### Trigonometry

All trigonometric functions work in **radians** and accept both `atom` and `fractional`, always returning `fractional`.

| Salm | Returns | Description |
|------|---------|-------------|
| `sine` | `fractional` | Sine |
| `cos` | `fractional` | Cosine |
| `tan` | `fractional` | Tangent |
| `asin` | `fractional` | Arcsine (result in radians) |
| `acos` | `fractional` | Arccosine (result in radians) |
| `atan` | `fractional` | Arctangent (result in radians) |
| `atan2` | `fractional` | Arctangent of `y/x` preserving quadrant (`atan2(y, x)`) |

### Logarithms

| Salm | Returns | Description |
|------|---------|-------------|
| `ln` | `fractional` | Natural logarithm (base *e*) |
| `log2` | `fractional` | Logarithm base 2 |
| `log10` | `fractional` | Logarithm base 10 |

### Standard library — `arithmos`

Requires `testament arithmos`. Provides constants and angle converters. See [Modules](modules.md#standard-library--arithmos) for details.

| Salm | Returns | Value / Description |
|------|---------|---------------------|
| `pi` | `fractional` | 3.141592653589793 |
| `euler` | `fractional` | 2.718281828459045 (*e*) |
| `tau` | `fractional` | 6.283185307179586 |
| `to_degrees(rad)` | `fractional` | Radians → degrees |
| `to_radians(deg)` | `fractional` | Degrees → radians |

### Collections

| Salm | Returns | Description |
|------|---------|-------------|
| `legion` | `legion of T` | Creates a legion from the given arguments |

```holy
-- I/O
hail proclaim praying "Hello, world!"
hail herald   praying "no newline"
let there line of word be hail inquire
let there a    of legion of word be hail args

-- file
let there r of verdict of word and word be hail read_file praying "data.txt"
hail write_file praying "output.txt" and "content"

-- exit
hail exit praying 0

-- conversion
let there n  of atom         be hail atom_of praying "42"
let there r  of verdict of atom and word be hail parse_atom praying "abc"
let there f  of fractional   be hail fractional_of praying 7
let there s  of word         be hail word_of praying 3.14
let there xs of legion of atom be hail legion praying 1, 2 and 3

-- math
hail proclaim praying hail word_of praying hail abs praying negate 5   -- 5
hail proclaim praying hail word_of praying hail floor praying 3.9      -- 3
hail proclaim praying hail word_of praying hail pow praying 2 and 10   -- 1024
hail proclaim praying hail word_of praying hail min praying 4 and 9    -- 4
hail proclaim praying hail word_of praying hail sqrt praying 2.0       -- 1.4142135623730951

-- trigonometry (radians)
let there pi of fractional be 3.141592653589793
hail proclaim praying hail word_of praying hail sine praying pi over 2   -- 1.0
hail proclaim praying hail word_of praying hail cos praying 0            -- 1.0
hail proclaim praying hail word_of praying hail tan praying pi over 4    -- ~1.0
hail proclaim praying hail word_of praying hail atan2 praying 1.0 and 1.0  -- ~0.7854 (pi/4)

-- logarithms
hail proclaim praying hail word_of praying hail ln praying 2.718281828  -- ~1.0
hail proclaim praying hail word_of praying hail log2 praying 8.0        -- 3.0
hail proclaim praying hail word_of praying hail log10 praying 1000.0    -- 3.0
```

`proclaim` and `herald` expect a `word`. To print other types, convert with `word_of` first:

```holy
let there x of atom be 42
hail proclaim praying hail word_of praying x
```
