# Sins

Sins are the exception mechanism of Holy Lang. A sin is thrown with `transgress` and caught with `confess`/`answer for`.

---

## Declaring a sin

```holy
sin Failure
    message of word

sin OutOfBounds
    index   of atom
    max     of atom
```

A sin with no fields is valid — the type alone is enough:

```holy
sin NotFound
```

---

## Throwing — `transgress`

```holy
transgress Failure praying "something went wrong"
transgress OutOfBounds praying index, max
transgress OutOfBounds praying index and max
transgress NotFound
```

Arguments are in field declaration order. As with other Holy lists, the final separator may be `and`. `praying` is omitted if the sin has no fields.

`transgress` immediately halts execution of the current block and propagates up the call stack until caught or the program terminates.

---

## Catching — `confess` / `answer for` / `absolve`

```holy
confess
    -- try block
    transgress Failure praying "oops"
answer for Failure
    -- handler: sin type matched but instance not bound
    hail proclaim praying "a failure occurred"
answer for OutOfBounds as err
    -- 'as name' binds the sin instance (it's a scripture-like value)
    hail proclaim praying "out of bounds: index " plus hail word_of praying index from err
absolve
    -- finally block: always runs, with or without an error
    hail proclaim praying "cleanup"
```

Rules:
- `confess` opens the try block.
- At least one `answer for SinType` clause is required.
- `as name` is optional; use it when you need to access the sin's fields.
- `absolve` (finally) is optional and appears last.
- Multiple `answer for` clauses match different sin types; the first match wins.
- If no clause matches, the sin propagates further up the stack.

---

## Accessing sin fields

Inside an `answer for … as name` block, the bound variable behaves like a scripture instance:

```holy
sin ParseError
    input   of word
    column  of atom

confess
    transgress ParseError praying "abc" and 3
answer for ParseError as e
    hail proclaim praying "bad input: " plus input from e
    hail proclaim praying "at column: " plus hail word_of praying column from e
```

---

## Built-in sins

The runtime raises these sins automatically for common errors. They can be caught with `answer for`:

| Sin name                  | When raised |
|---------------------------|-------------|
| `DivisionByZero`          | `a over 0` or `a remainder 0` |
| `TypeError`               | value does not match declared type |
| `InvalidArgumentCount`    | wrong number of arguments to a salm or scripture |
| `UndefinedVariable`       | variable not declared in scope |
| `UndefinedSalm`           | `hail` of an undeclared salm name |
| `UndefinedField`          | `from` on a non-existent field |
| `UndefinedSin`            | `transgress` of an undeclared sin |
| `UndefinedType`           | type annotation refers to an unknown type |
| `InvalidDiscern`          | `discern` on a non-covenant value, or no branch matched |
| `InvalidContext`          | `its` used outside a method salm |

```holy
confess
    let there n of atom be hail atom_of praying "not a number"
    let there result of atom be 100 over n
answer for DivisionByZero
    hail proclaim praying "cannot divide by zero"
answer for TypeError as e
    hail proclaim praying "type error"
```

---

## Sin propagation

If a sin is not caught anywhere in the call stack, the program terminates with an error message:

```
error: unhandled sin: Failure (something went wrong)
```
