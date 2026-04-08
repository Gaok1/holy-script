# Holy Script
<img width="500" height="500" alt="Emblema da linguagem Holy Script" src="https://github.com/user-attachments/assets/c6a7e8f8-0add-4c8b-bd4b-2d47463baa83" />

An interpreted, strongly typed programming language with archaic/biblical syntax, implemented in Rust.

Types are declared explicitly and enforced at runtime. Variables, scripture fields, salm parameters, and return values must match their declared types.

Across Holy's list syntax, the final separator may be written as `and`: `a and b`, `a, b and c`, `Pair of atom and word`, `receiving x of atom and y of atom`.

```holy
scripture Person
    name of word
    age  of atom

salm greet upon Person reveals void
    hail proclaim praying "Hail, " plus name from its plus "!"

let there p of Person be manifest Person praying "Gabriel", 30
hail greet upon p

amen
```

---

## Documentation

| Topic | Description |
|-------|-------------|
| [Types & Variables](docs/types.md) | Primitive types, literals, variables, operators, expression grouping |
| [Scriptures](docs/scriptures.md) | Struct-like data types, field access, method salms |
| [Covenants](docs/covenants.md) | Sum types, pattern matching with `discern`, built-in `grace` and `verdict` |
| [Salms](docs/salms.md) | Functions, parameters, return values, built-in salms |
| [Control Flow](docs/control-flow.md) | `whether`, `litany for`, `forsake`, `ascend` |
| [Sins](docs/sins.md) | Exception types, `transgress`, `confess`/`answer for`/`absolve` |
| [Generics](docs/generics.md) | Type parameters, `thus` disambiguation, generic calls |
| [Nesting](docs/nesting.md) | Disambiguation with `thus` and `after` for nested calls, generics, and expressions |
| [Modules](docs/modules.md) | `testament` imports, `revealing` selective imports |

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

## CLI

```bash
holy <file.holy>          # execute a program
holy --tree <file.holy>   # print the parse tree (does not execute)
holy -t <file.holy>       # same as --tree
```

---

## Program structure

Every program has three sections in order:

```holy
-- 1. module imports (optional)
testament MathUtils
testament Collections revealing Stack and Queue

-- 2. top-level declarations (in any order)
scripture Point
    x of atom
    y of atom

sin OutOfBounds

covenant Direction
    North
    South

salm add receiving a of atom and b of atom reveals atom
    reveal a plus b

-- 3. top-level statements
let there p of Point be manifest Point praying 3 and 4

amen       -- required: marks the end of the program
```

Every program **must** end with `amen`. Comments start with `--`.

---

## Quick reference

### Variables
```holy
let there be x of atom          -- declare, zero-initialised
let there name of word be "Hi"  -- declare with value
x become 99                     -- reassign
```

For `scripture` values, reassignment also happens at the whole-value level: fields are readable with `from`, but inner fields are not directly mutable. To update one field, create a new `manifest ...` value and assign it back to the variable.

### Operators
```holy
a plus b   a minus b   a times b   a over b   a remainder b   negate a
a is b     a is not b
a greater than b    a lesser than b
a no greater than b   a no lesser than b
```

### Expression grouping
```holy
after 3 times 5 thus        -- (3 * 5) = 15
5 plus after 3 times 2 thus -- 5 + (3 * 2) = 11
```

### Functions
```holy
salm double receiving n of atom reveals atom
    reveal n times 2

let there x of atom be hail double praying 7
```

### Conditionals & loops
```holy
whether x greater than 0
    hail proclaim praying "positive"
otherwise
    hail proclaim praying "non-positive"

litany for i no greater than 10
    i become i plus 1
```

### Pattern matching
```holy
discern result
    as righteous bearing value
        hail proclaim praying hail word_of praying value
    as condemned bearing reason
        hail proclaim praying reason
```

---

## Reserved words

```holy
testament  revealing  scripture  sin  covenant  salm  upon  receiving  reveals
let  there  be  of  become  hail  praying  reveal  whether  otherwise  so
litany  for  forsake  ascend  bearing  confess  answer  absolve  as  transgress
manifest  from  its  discern  amen  negate  remainder  after  thus
plus  minus  times  over  is  not  greater  lesser  than  no
blessed  forsaken  and  void  atom  fractional  word  dogma
grace  granted  absent  verdict  righteous  condemned
```
