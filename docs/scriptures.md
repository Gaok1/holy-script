# Scriptures

Scriptures are data structures — named collections of typed fields. Think of them as `struct` in other languages. A scripture only holds data; behaviour is added through [method salms](#method-salms).

---

## Declaring a scripture

```holy
scripture Point
    x of atom
    y of atom

scripture Person
    name of word
    age  of atom
```

- At least one field is required.
- Fields are declared in order; that order matters when creating values.
- Field names must be unique within the scripture.

---

## Creating a value — `manifest`

```holy
let there p of Point  be manifest Point  praying 3, 4
let there u of Person be manifest Person praying "Gabriel", 30
```

Arguments are passed **in field declaration order**, separated by `,`. The final separator may be `and`:

```holy
let there p of Point be manifest Point praying 3 and 4
```

---

## Reading fields — `from`

```holy
let there px of atom be x from p
let there nm of word be name from u
```

`from` reads a field by name. It does not change the value.

Fields can be chained when a field is itself a scripture:

```holy
scripture Address
    city of word

scripture Employee
    name    of word
    address of Address

let there emp of Employee be manifest Employee praying "Ava", manifest Address praying "London"

let there city of word be city from address from emp
-- reads emp.address.city
```

---

## Value semantics (immutability)

Scriptures in Holy are **immutable values** — you cannot assign directly to a field. To "change" a scripture, create a new value and reassign the variable:

```holy
scripture Person
    name of word
    age  of atom

let there p of Person be manifest Person praying "Gabriel", 30

-- not allowed: direct field assignment does not exist
-- age from p become 31   ← syntax error

-- correct: create a new value
p become manifest Person praying name from p, 31
```

The same applies to nested scriptures: build a new inner value, then a new outer value, and reassign.

---

## Method salms

A method salm is declared with `upon TargetType` and is called on an instance of that type:

```holy
salm introduce upon Person reveals void
    hail proclaim praying "I am " plus name from its

-- call
hail introduce upon p
```

With parameters:

```holy
salm greetWith upon Person receiving greeting of word reveals void
    hail proclaim praying greeting plus ", " plus name from its plus "!"

hail greetWith upon p praying "Hail"
```

### `its` — the current instance

Inside a method salm body, `its` refers to the instance the method was called on:

```holy
salm fullName upon Person reveals word
    reveal name from its plus " (age " plus hail word_of praying age from its plus ")"

let there fn of word be hail fullName upon p
hail proclaim praying fn    -- "Gabriel (age 31)"
```

- `its` has the type of the scripture bound to the method.
- `its` is read-only; it cannot be reassigned.

---

## Constructor convention

By convention, a salm with the same name as the scripture acts as a constructor:

```holy
scripture Point
    x of atom
    y of atom

salm Point receiving x of atom, y of atom reveals Point
    reveal manifest Point praying x, y

-- cleaner usage
let there p of Point be hail Point praying 3, 4
```

---

## Generic scriptures

Scriptures can declare type parameters with `of`:

```holy
scripture Box of T
    value of T

scripture Pair of A and B
    first  of A
    second of B
```

When creating a value, pass the types explicitly:

```holy
let there b of Box of atom           be manifest Box  praying 42
let there p of Pair of atom and word be manifest Pair praying 1 and "x"
```

See [Generics](generics.md) for the full rules on generic types and use of `thus`.

---

## Default value

When declared without an initial value (`let there be`), a scripture variable is initialised as `void`. Accessing its fields before assigning a real value causes a runtime error:

```holy
let there be p of Point         -- p = void internally
p become manifest Point praying 0, 0   -- safe now
let there px of atom be x from p       -- 0
```

---

## Full example

```holy
scripture Rectangle
    width  of fractional
    height of fractional

salm area upon Rectangle reveals fractional
    reveal width from its times height from its

salm describe upon Rectangle reveals void
    let there a of fractional be hail area upon its
    hail proclaim praying "Area: " plus hail word_of praying a

let there r of Rectangle be manifest Rectangle praying 5.0 and 3.0
hail describe upon r    -- "Area: 15"

amen
```
