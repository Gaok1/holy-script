# Holy Script

An interpreted programming language with archaic/biblical English syntax, implemented in Rust.

---

## Installation

```bash
git clone <repo>
cd holy-script

# run directly
cargo run -- program.holy

# install globally
cargo install --path .
holy program.holy
```

---

## CLI

```bash
holy <file.holy>          # execute a program
holy --tree <file.holy>   # print the parse tree (does not execute)
holy -t <file.holy>       # same as --tree
```

---

## Program structure

Every program begins with top-level declarations (`scripture`, `sin`, `covenant`, `salm`) and **must** end with exactly one `amen`.

```
-- line comment

scripture ...
sin ...
covenant ...
salm ...

-- top-level statements

amen
```

---

## Types

| Keyword      | Type              | Literal example          |
|--------------|-------------------|--------------------------|
| `atom`       | integer (i64)     | `42`, `-7`               |
| `fractional` | float (f64)       | `3.14`, `-0.5`           |
| `word`       | string            | `"hello"`                |
| `dogma`      | bool              | `blessed` / `forsaken`   |
| `void`       | no return value   | —                        |
| `CustomName` | user scripture    | —                        |

---

## Variables

```
-- declare without a value (zero-initialised to the type's default)
let there be x of atom

-- declare with a value
let there name of word be "Gabriel"

-- reassign
x become 42
```

---

## Operators

### Arithmetic
```
a plus b        -- addition / string concatenation
a minus b       -- subtraction
a times b       -- multiplication
a over b        -- division
a remainder b   -- modulo (atom only)
negate a        -- unary minus
```

### Comparison
```
a is b              -- ==
a is not b          -- !=
a greater than b    -- >
a lesser than b     -- <
a no greater than b -- <=
a no lesser than b  -- >=
```

> Numeric comparisons work with `atom` and `fractional`.  
> `is` / `is not` work with any type.

---

## Scriptures (structs)

```
scripture Person
    name of word
    age  of atom

-- instantiate (args in field declaration order)
let there p of Person be manifest Person praying "Ana", 30

-- field access
let there n of word be name from p
```

---

## Sins (exceptions)

```
sin Failure
    message of word

-- a sin with no fields is also valid
sin SimpleFailure
```

---

## Covenants (enums)

```
covenant Direction
    North
    South
    East
    West

let there d of Direction be North
```

---

## Salms (functions)

```
salm add receiving a of atom, b of atom reveals atom
    reveal a plus b

-- call
let there result of atom be hail add praying 3, 5

-- no parameters
salm greet reveals void
    hail proclaim praying "Hail!"
```

---

## Method Salms

Bound to a scripture via `upon`. Inside the body, `its` references the instance.

```
salm introduce upon Person reveals void
    hail proclaim praying name from its

-- call
hail introduce upon p
```

---

## Constructor convention

A salm with the same name as a scripture that returns `manifest`:

```
salm Person receiving name of word, age of atom reveals Person
    reveal manifest Person praying name, age

let there p of Person be hail Person praying "Ana", 30
```

---

## Conditional

```
whether x greater than 10
    hail proclaim praying "large"
otherwise so x is 10
    hail proclaim praying "exact"
otherwise
    hail proclaim praying "small"
```

---

## Loop

Executes while the condition is truthy (`blessed`).

```
let there be i of atom
i become 1
litany for i no greater than 5
    hail proclaim praying hail word_of praying i
    i become i plus 1
```

### Loop control

```
forsake   -- break: exits the litany immediately
ascend    -- continue: jumps to the next iteration
```

---

## Confess (try / catch / finally)

```
sin Problem
    description of word

confess
    transgress Problem praying "something went wrong"
answer for Problem as err
    hail proclaim praying description from err
absolve
    hail proclaim praying "always runs"
```

- `confess` → try block
- `answer for SinType` → catch (one clause per type)
- `as name` → optional, binds the sin instance to a variable
- `absolve` → finally (optional)

---

## Discern (pattern matching)

```
covenant Status
    Active
    Inactive
    Banned

let there s of Status be Active

discern s
    as Active
        hail proclaim praying "online"
    as Inactive
        hail proclaim praying "away"
    otherwise
        hail proclaim praying "banned"
```

---

## Built-in salms

| Salm       | Description                              |
|------------|------------------------------------------|
| `proclaim` | print with newline                       |
| `herald`   | print without newline                    |
| `inquire`  | read a line from stdin → `word`          |
| `atom_of`  | convert `word` → `atom`                 |
| `word_of`  | convert any value → `word`              |

---

## Full example

```
scripture Apostate
    name  of word
    age   of atom
    heretic of dogma

sin HeresyDetected
    reason of word

salm Apostate receiving name of word, age of atom, heretic of dogma reveals Apostate
    reveal manifest Apostate praying name, age, heretic

salm judge upon Apostate reveals void
    whether heretic from its
        transgress HeresyDetected praying name from its plus " is a heretic"
    otherwise
        hail proclaim praying name from its plus " is absolved"

let there a of Apostate be hail Apostate praying "John", 33, blessed

confess
    hail judge upon a
answer for HeresyDetected as err
    hail proclaim praying reason from err
absolve
    hail proclaim praying "judgment concluded"

amen
```

---

## Reserved words

`testament` `revealing` `scripture` `sin` `covenant` `salm` `upon` `receiving` `reveals`
`let` `there` `be` `of` `become` `hail` `praying` `reveal` `whether` `otherwise`
`so` `litany` `for` `forsake` `ascend` `confess` `answer` `absolve` `as` `transgress`
`manifest` `from` `its` `discern` `amen` `negate` `remainder`
`plus` `minus` `times` `over` `is` `not` `greater` `lesser`
`than` `no` `blessed` `forsaken` `and` `void` `atom` `fractional` `word` `dogma`
