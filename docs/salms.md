# Salms

Salms are functions. They optionally receive parameters, always declare a return type, and return a value with `reveal`.

---

## Declaration

```holy
salm add receiving a of atom, b of atom reveals atom
    reveal a plus b

salm greet reveals void
    hail proclaim praying "Hail!"
```

- `receiving param_list` is optional (omit if no parameters).
- `reveals type` is required; use `void` when the salm produces no value.
- The body is an indented block with at least one statement.
- Lists in Holy may use `and` for the final separator: `a and b`, `a, b and c`.

---

## Parameters

```holy
salm describe receiving name of word, age of atom, active of dogma reveals word
    reveal name plus " (" plus hail word_of praying age plus ")"
```

There is no variadic syntax. Each parameter has an explicit name and type.

The final parameter separator may also be `and`:

```holy
salm describe receiving name of word, age of atom and active of dogma reveals word
    reveal name
```

---

## Return value — `reveal`

`reveal expr` returns a value and exits the salm immediately.

```holy
salm max receiving a of atom, b of atom reveals atom
    whether a greater than b
        reveal a
    reveal b
```

A `void` salm may omit `reveal` entirely (falls off the end) or use `reveal` with no expression — but the simplest practice is to just let the block end naturally.

---

## Calling a salm — `hail`

```holy
-- no arguments
hail greet

-- with arguments
let there sum of atom be hail add praying 3, 5

-- final separator may use 'and' instead of ','
let there sum of atom be hail add praying 3 and 5
```

### Nested calls

A salm call can be used as an argument to another call:

```holy
hail proclaim praying hail word_of praying 42
```

When a nested call is not the last argument, use `thus` to close it so the `and` belongs to the outer call:

```holy
-- add(double(3), 1) = 7
let there y of atom be hail add praying hail double praying 3 thus and 1
```

Without `thus`, `and 1` would be consumed as a second argument to `double`. See [Disambiguation with `thus` and `after`](nesting.md#2-disambiguating-nested-calls) and [Generics — `thus`](generics.md#thus--disambiguation).

---

## Generic salms

Salms can declare type parameters with `of` after their name:

```holy
salm identity of T receiving val of T reveals T
    reveal val

salm wrap of T receiving val of T reveals grace of T
    reveal manifest granted of grace of T praying val
```

Type args are passed explicitly at the call site:

```holy
let there g of grace of atom be hail wrap of atom praying 42
let there w of grace of word be hail wrap of word praying "hello"
```

Type parameters are erased at runtime — the interpreter accepts any value for an abstract type param without enforcing the type. Concrete types (`atom`, `word`, registered scriptures/covenants) are still checked.

---

## Method salms

Declared with `upon TypeName`. See [Scriptures — Method salms](scriptures.md#method-salms).

```holy
salm area upon Rectangle reveals fractional
    reveal width from its times height from its

hail area upon rect
```

---

## `reveal` as an expression

`reveal` terminates the current salm. It can appear anywhere inside the body, including inside a `whether` or `discern` branch.

```holy
salm sign receiving n of atom reveals word
    whether n greater than 0
        reveal "positive"
    whether n lesser than 0
        reveal "negative"
    reveal "zero"
```

---

## Built-in salms

These salms are available in every program without declaration.

| Salm       | Signature                        | Description                          |
|------------|----------------------------------|--------------------------------------|
| `proclaim` | `receiving val of word → void`   | Print `val` followed by a newline    |
| `herald`   | `receiving val of word → void`   | Print `val` without a newline        |
| `inquire`  | `→ word`                         | Read a line from stdin               |
| `atom_of`  | `receiving val of word → atom`   | Parse a `word` as an integer         |
| `word_of`  | `receiving val of any → word`    | Convert any value to its string form |

All must be called with `hail`:

```holy
hail proclaim praying "Hello, world!"
hail herald praying "no newline"
let there line of word be hail inquire
let there n of atom be hail atom_of praying line
let there s of word be hail word_of praying 3.14
```

`proclaim` and `herald` accept a `word`; pass any other type through `word_of` first:

```holy
let there x of atom be 42
hail proclaim praying hail word_of praying x
```

---

## Salm as a statement

A salm call can stand alone as a statement (ignoring the return value):

```holy
hail proclaim praying "done"
hail sort upon list
```
