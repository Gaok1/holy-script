# Generics

Generics allow you to write scriptures, covenants, and salms that work with **any type**, without duplicating code. The concrete type is specified at each use site.

In Holy everything is **explicit** — there is no type inference. Type arguments are always written at each call and instantiation.

---

## Declaring type parameters

Use `of` followed by parameter names after the declaration name. The final separator may be `and`:

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

Parameter names are conventional — uppercase letters by convention (`T`, `E`, `A`, `B`, …).

---

## Passing type arguments

At each call or instantiation, pass the types explicitly with `of`:

```holy
-- instantiating scriptures
let there b of Box of atom           be manifest Box  praying 42
let there p of Pair of atom and word be manifest Pair praying 1 and "x"

-- calling salms
let there x of atom          be hail identity of atom praying 99
let there g of grace of atom be hail wrap of atom praying 42

-- instantiating covenant variants
let there o of Option of atom be manifest Some of Option of atom praying 7
let there n of Option of atom be None of Option of atom
```

---

## `thus` — disambiguation

`thus` is a **close marker**. It signals to the parser that the argument list of a nested generic type has ended, so the next `,` or `and` belongs to the outer context.

### The problem without `thus`

The parser greedily consumes all `,` after a type, treating them as more arguments for the innermost type being parsed:

```holy
-- WRONG: parser reads "Stack<T, word>" → verdict has no second argument
verdict of Stack of T, word

-- WRONG: same issue
verdict of Stack of atom, word
```

### The solution — `thus`

```holy
-- CORRECT: thus closes Stack<T>, then "and word" goes to verdict
verdict of Stack of T thus and word

-- CORRECT: thus closes Stack<atom>
verdict of Stack of atom thus and word
```

### Simple types never need `thus`

```holy
verdict of atom and word     -- ok: atom has no type arguments
grace of word                -- ok
verdict of T and E           -- ok: T and E are plain names without "of"
```

`thus` is only needed when a type argument is itself generic **and** is followed by a separator that belongs to the outer context.

---

## Where `thus` appears

### 1. Type annotations

In any position where a type is written — `let there`, `reveals`, `receiving`, fields:

```holy
-- return type
salm pop of T receiving s of Stack of T reveals verdict of Stack of T thus and word
    -- ...

-- parameter: "s of Stack<T>" then "and val of T" is the next param
salm push of T receiving s of Stack of T thus and val of T reveals Stack of T
    -- ...

-- variable declaration
let there result of verdict of Stack of atom thus and word be hail pop of atom praying s
```

### 2. Variant instantiation

When the type argument of a covenant/variant is itself generic:

```holy
-- righteous carries Stack<T>, E is word
manifest righteous of verdict of Stack of T thus and word praying newStack

-- granted carries StackNode<T> (thus not needed: single argument of grace)
manifest granted of grace of StackNode of T praying node
```

### 3. Nested call arguments

When a call is used as an argument of another and is **not the last argument**, `thus` closes the inner argument list:

```holy
-- add(double(3), 1) — thus closes double's args before "and 1"
hail add praying hail double praying 3 thus and 1

-- a(b(c(1)), 2) — first thus closes c, second thus closes b
hail a praying hail b praying hail c praying 1 thus thus and 2
```

Without `thus`, `and 1` would be parsed as a second argument of `double`.

### 4. Expression grouping — `after`

`after` deepens the parser into full expression parsing. `thus` is **optional**: it closes the group early when the outer expression needs to continue after it.

```holy
after 3 times 5              -- (3 * 5) = 15  (no thus)
5 plus after 3 times 2       -- 5 + (3 * 2) = 11  (no thus)
after a plus b thus times c  -- (a + b) * c  (thus needed here)
```

See [Nesting](nesting.md) for all disambiguation cases.

---

## `thus` rules

Each `thus` closes exactly **one** open context. Possible contexts:

| Opened by | Closed by |
|-----------|-----------|
| `of` in a generic type argument | `thus` inside type parsing |
| `praying` in a call | `thus` after the argument list |
| `after` | optional `thus` — closes the group early if present |

A `thus` with no matching open context is a syntax error.

---

## Runtime type erasure

Type parameters are erased at runtime. The interpreter does **not** check generic constraints for:

- User-defined scriptures and covenants with type parameters
- Salm parameters typed with an abstract parameter (e.g. `T`)

It **does** check concrete types:

```holy
-- TypeError: granted of grace of atom expects atom, received word
manifest granted of grace of atom praying "hello"

-- OK: T is abstract, no check
salm identity of T receiving val of T reveals T
    reveal val

hail identity of atom praying "any value"   -- runtime: accepted
```

The built-in `grace` and `verdict` have runtime type checking — their concrete type arguments are checked at instantiation.

---

## Full example — generic stack

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
            reveal manifest condemned of verdict of T and word praying "empty stack"

let there s of Stack of atom be hail emptyStack of atom
s become hail push of atom praying s and 10
s become hail push of atom praying s and 20

let there peeked of verdict of atom and word be hail peek of atom praying s
discern peeked
    as righteous bearing value
        hail proclaim praying hail word_of praying value   -- "20"
    as condemned bearing reason
        hail proclaim praying reason

amen
```
