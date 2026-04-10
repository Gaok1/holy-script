# Control Flow

Holy provides conditionals with `whether` and loops with `litany for`. There is no iterator-based `for` or `do-while` — only the conditional loop.

---

## Conditional — `whether`

```holy
whether condition
    -- block executed if true
```

```holy
let there x of atom be 7

whether x greater than 10
    hail proclaim praying "large"
otherwise so x is 10
    hail proclaim praying "exactly ten"
otherwise
    hail proclaim praying "small"
```

- `whether` evaluates the condition. If true, the block below is executed.
- `otherwise so condition` is an else-if (zero or more).
- `otherwise` is the final else (optional, at most one).
- The first true branch is executed; the rest are skipped.

### Truthy and falsy values

| Type         | Truthy when              |
|--------------|--------------------------|
| `dogma`      | `blessed`                |
| `atom`       | not equal to `0`         |
| `fractional` | not equal to `0.0`       |
| `word`       | non-empty string         |
| `void`       | never                    |
| scripture    | always                   |
| covenant     | always                   |

---

## Loop — `litany for`

Repeats the body while the condition is true.

```holy
let there i of atom be 1

litany for i no greater than 5
    hail proclaim praying hail word_of praying i
    i become i plus 1
```

Output: `1`, `2`, `3`, `4`, `5` (one per line).

The condition is evaluated before each iteration. If it is false on the first check, the body never runs.

**Infinite loop** — use `forsake` to exit:

```holy
litany for blessed
    -- runs forever until a forsake
```

---

## Loop control

### `forsake` — break

Exits the `litany for` immediately.

```holy
let there i of atom be 1

litany for blessed
    whether i is 5
        forsake
    hail proclaim praying hail word_of praying i
    i become i plus 1
```

Output: `1`, `2`, `3`, `4`

### `ascend` — continue

Skips the rest of the current iteration and jumps back to the condition check.

```holy
let there i of atom be 0

litany for i lesser than 10
    i become i plus 1
    whether i remainder 2 is 0
        ascend          -- skip even numbers
    hail proclaim praying hail word_of praying i
```

Output: `1`, `3`, `5`, `7`, `9`

`forsake` and `ascend` are only valid inside a `litany for`. Using them outside a loop causes a runtime error.

---

## Nested loops

`forsake` and `ascend` affect only the **innermost** `litany for`:

```holy
let there i of atom be 1

litany for i no greater than 3
    let there j of atom be 1
    litany for j no greater than 3
        whether j is 2
            forsake             -- exits only the inner loop
        hail proclaim praying hail word_of praying i plus "," plus hail word_of praying j
        j become j plus 1
    i become i plus 1
```

Output:
```
1,1
2,1
3,1
```

---

## Full example — FizzBuzz

```holy
let there i of atom be 1

litany for i no greater than 20
    whether i remainder 15 is 0
        hail proclaim praying "FizzBuzz"
    otherwise so i remainder 3 is 0
        hail proclaim praying "Fizz"
    otherwise so i remainder 5 is 0
        hail proclaim praying "Buzz"
    otherwise
        hail proclaim praying hail word_of praying i
    i become i plus 1

amen
```

---

## Full example — summing a legion

```holy
let there nums of legion of atom be hail legion praying 10, 20, 30, 40 and 50
let there sum  of atom be 0
let there i    of atom be 0

litany for i lesser than hail length upon nums
    sum become sum plus hail at upon nums praying i
    i become i plus 1

hail proclaim praying hail word_of praying sum   -- 150

amen
```
