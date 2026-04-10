# Sins — Exceptions

Sins are Holy's exception mechanism. A sin is thrown with `transgress` and caught with `confess`/`answer for`. If uncaught, the program exits with an error message and a stack trace.

---

## Declaring a sin

```holy
sin Failure
    message of word

sin OutOfBounds
    index of atom
    max   of atom
```

A sin with no fields is also valid — the type alone is sufficient information:

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

- Arguments follow the **field declaration order**.
- The final separator may be `and`.
- `praying` is omitted when the sin has no fields.

`transgress` interrupts execution of the current block and propagates the sin up the call stack until it is caught or terminates the program.

---

## Catching — `confess` / `answer for` / `absolve`

```holy
confess
    -- try block
    transgress Failure praying "oops"
answer for Failure
    -- no binding: only knows the type
    hail proclaim praying "a failure occurred"
answer for OutOfBounds as err
    -- 'as name' binds the sin to the variable err
    hail proclaim praying "out of bounds: index " plus hail word_of praying index from err
absolve
    -- finally block: always runs, with or without an error
    hail proclaim praying "cleanup"
```

Rules:
- `confess` opens the try block.
- At least one `answer for SinType` is required.
- `as name` is optional; use it when you need to access the sin's fields.
- `absolve` (finally) is optional and comes last.
- Multiple `answer for` handlers cover different types; the first matching one is executed.
- If none match, the sin continues propagating up the stack.

---

## Accessing sin fields

Inside an `answer for … as name` block, the bound variable behaves like a scripture:

```holy
sin ParseError
    input  of word
    column of atom

confess
    transgress ParseError praying "abc" and 3
answer for ParseError as e
    hail proclaim praying "invalid input: " plus input from e
    hail proclaim praying "at column: " plus hail word_of praying column from e
```

---

## Built-in sins

The runtime throws these sins automatically for common errors. All of them can be caught with `answer for`:

| Sin name                  | When thrown |
|---------------------------|-------------|
| `DivisionByZero`          | `a over 0` or `a remainder 0` |
| `TypeError`               | value does not match the declared type |
| `InvalidArgumentCount`    | wrong number of arguments |
| `UndefinedVariable`       | variable not declared in scope |
| `UndefinedSalm`           | `hail` of an undeclared salm |
| `UndefinedField`          | `from` on a non-existent field |
| `IndexOutOfBounds`        | `at` on `word` or `legion` with an invalid index |
| `UndefinedSin`            | `transgress` of an undeclared sin |
| `UndefinedType`           | type annotation references an unknown type |
| `InvalidDiscern`          | `discern` on a non-covenant value, or no branch matched |
| `InvalidContext`          | `its` used outside a method salm |

```holy
confess
    let there n of atom be hail atom_of praying "not a number"
    let there result of atom be 100 over n
answer for DivisionByZero
    hail proclaim praying "cannot divide by zero"
answer for TypeError
    hail proclaim praying "invalid type"
```

---

## Sin propagation

If a sin is not caught anywhere in the call stack, the program exits with an error message and a call trace showing which salms the sin passed through:

```
O Profanation!: an unabsolved sin has escaped into the world: Failure — something went wrong
    at safeDivide
    at compute
```

---

## Full example

```holy
sin DivisionError
    message of word

salm safeDivide receiving a of atom, b of atom reveals atom
    whether b is 0
        transgress DivisionError praying "divisor cannot be zero"
    reveal a over b

confess
    let there result of atom be hail safeDivide praying 10, 0
    hail proclaim praying hail word_of praying result
answer for DivisionError as e
    hail proclaim praying "Error: " plus message from e
absolve
    hail proclaim praying "end of operation"

amen
```

Output:
```
Error: divisor cannot be zero
end of operation
```
