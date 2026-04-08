# Generics

Holy Lang supports type parameters on scriptures, covenants, and salms. Everything is **explicit** — there is no type inference. Type arguments are always written out at every call and instantiation site.

---

## Declaring type parameters

Use `of` followed by a comma-separated list of type parameter names after the declaration name. The final separator may also be `and`.

```holy
-- generic scripture
scripture Box of T
    value of T

scripture Pair of A and B
    first  of A
    second of B

-- generic covenant
covenant Option of T
    Some
        value of T
    None

-- generic salm
salm identity of T receiving val of T reveals T
    reveal val

salm wrap of T receiving val of T reveals grace of T
    reveal manifest granted of grace of T praying val
```

Type parameter names are conventional identifiers (single capital letters by convention: `T`, `E`, `A`, `B`, …).

---

## Passing type arguments

At every call or instantiation site, type args are supplied explicitly with `of`. The final separator may also be `and`:

```holy
-- scripture instantiation
let there b of Box of atom be manifest Box praying 42
let there p of Pair of atom and word be manifest Pair praying 1 and "x"

-- salm call
let there x of atom be hail identity of atom praying 99
let there g of grace of atom be hail wrap of atom praying 42

-- variant instantiation
let there o of Option of atom be manifest Some of Option of atom praying 7
let there n of Option of atom be None of Option of atom
```

---

## `thus` — disambiguation

`thus` is a **closing marker**. It signals to the parser that a nested generic type's argument list is complete, so the next `,` belongs to the outer context instead.

For a focused guide to ambiguity cases across generic types, nested calls, and grouped expressions, see [Disambiguation with `thus` and `after`](nesting.md).

### The problem without `thus`

```holy
-- WRONG: parser reads "Stack<T, word>" then verdict has no second arg → error
verdict of Stack of T, word

-- WRONG: parser reads "Stack<atom, word>" same problem
verdict of Stack of atom, word
```

The parser greedily consumes every `,` that follows a type, treating it as another type argument for the innermost generic type being parsed.

### The solution — `thus`

```holy
-- CORRECT: thus closes Stack<T>, then "and word" goes to verdict
verdict of Stack of T thus and word

-- CORRECT: thus closes Stack<atom>
verdict of Stack of atom thus and word
```

### Simple types never need `thus`

```holy
verdict of atom and word     -- ok: atom has no type args, no ambiguity
grace of word                -- ok
verdict of T, E              -- ok: T and E are simple names with no "of"
```

`thus` is only needed when the type argument is itself generic **and** is followed by a separator that belongs to the outer context.

---

## Where `thus` appears

### 1. Type annotations

In any position where a type is written — `let there`, `salm reveals`, `receiving`, field declarations:

```holy
-- return type
salm pop of T receiving s of Stack of T reveals verdict of Stack of T thus and word
    ...

-- parameter type: "s of Stack<T>" then "and val of T" is the next param
salm push of T receiving s of Stack of T thus and val of T reveals Stack of T
    ...

-- variable declaration
let there result of verdict of Stack of atom thus and word be hail pop of atom praying s
```

### 2. Variant instantiation

When the type argument list of a covenant/variant is itself generic:

```holy
-- righteous carries Stack<T>, E is word
manifest righteous of verdict of Stack of T thus and word praying newStack

-- granted carries grace<StackNode<T>>
manifest granted of grace of StackNode of T praying node
-- (no thus needed here: StackNode<T> is the only type arg of grace, no following outer separator)
```

### 3. Nested `hail` / `manifest` argument lists

When a call is used as an argument to another call and is **not the last** argument, `thus` closes the inner call's argument list:

```holy
-- add(double(3), 1) — thus closes double's args before "and 1"
hail add praying hail double praying 3 thus and 1

-- a(b(c(1)), 2) — first thus closes c, second thus closes b
hail a praying hail b praying hail c praying 1 thus thus and 2
```

Without `thus`, `and 1` would be parsed as a second argument to `double`. See [Disambiguation with `thus` and `after`](nesting.md#2-disambiguating-nested-calls).

### 4. Expression grouping — `after … thus`

`after` opens a sub-expression and `thus` closes it, equivalent to parentheses:

```holy
after 3 times 5 thus            -- (3 * 5) = 15
5 plus after 3 times 2 thus     -- 5 + (3 * 2) = 11
```

See also [Disambiguation with `thus` and `after`](nesting.md#1-disambiguating-expression-precedence-with-after).

---

## `thus` context rules

Each `thus` pops exactly **one** open context. Contexts are:

| Opened by | Closed by |
|-----------|-----------|
| `of` in a generic type arg | `thus` inside `parse_type` |
| `praying` in a call | `thus` after the arg list |
| `after` | `thus` that follows the expression |

A `thus` with no matching open context is a syntax error.

---

## Type erasure at runtime

Type parameters are erased at runtime. The interpreter does **not** enforce generic constraints for:

- User-defined scriptures and covenants with type params
- Generic salm parameters typed with an abstract param (e.g. `T`)

It **does** enforce concrete types:

```holy
-- TypeError: granted of grace of atom expects atom, got word
manifest granted of grace of atom praying "hello"

-- OK: T is abstract, no check
salm identity of T receiving val of T reveals T
    reveal val
hail identity of atom praying "any value"   -- runtime: accepted
```

Built-in `grace` and `verdict` have first-class type enforcement — their concrete type args are checked at instantiation.

---

## Full example

```holy
scripture Stack of T
    top  of grace of StackNode of T
    size of atom

scripture StackNode of T
    value of T
    next  of grace of StackNode of T

salm emptyStack of T reveals Stack of T
    reveal manifest Stack praying absent of grace of StackNode of T and 0

salm push of T receiving s of Stack of T thus and val of T reveals Stack of T
    let there node of StackNode of T be manifest StackNode praying val and top from s
    reveal manifest Stack praying manifest granted of grace of StackNode of T praying node thus and size from s plus 1

salm peek of T receiving s of Stack of T reveals verdict of T and word
    discern top from s
        as granted bearing node
            reveal manifest righteous of verdict of T and word praying value from node
        as absent
            reveal manifest condemned of verdict of T and word praying "stack is empty"

let there s of Stack of atom be hail emptyStack of atom
s become hail push of atom praying s and 10
s become hail push of atom praying s and 20

let there peeked of verdict of atom and word be hail peek of atom praying s
discern peeked
    as righteous bearing value
        hail proclaim praying hail word_of praying value   -- 20
    as condemned bearing reason
        hail proclaim praying reason

amen
```
