# Collections — `legion of T`

`legion of T` is Holy's built-in typed collection. Think of it as an array where the element type is always declared explicitly.

---

## Creating a legion

Use the built-in `legion` salm with `hail`:

```holy
let there xs    of legion of atom be hail legion praying 1, 2 and 3
let there names of legion of word be hail legion praying "Ava" and "Noah"
```

Declaration without an initial value (empty legion):

```holy
let there be xs of legion of atom      -- [] (empty)
```

---

## Runtime typing

A `legion of atom` only accepts `atom` elements. Trying to insert another type throws `TypeError`:

```holy
-- OK
let there xs of legion of atom be hail legion praying 1 and 2
xs become hail push upon xs praying 3

-- TypeError: "oops" is not atom
let there ys of legion of atom be hail legion praying 1 and "oops"
```

---

## Available methods

All methods follow the standard syntax:

```holy
hail method upon target
hail method upon target praying arg1, arg2
```

| Method | Returns | Description |
|--------|---------|-------------|
| `hail length upon xs` | `atom` | number of elements |
| `hail is_empty upon xs` | `dogma` | whether the legion is empty |
| `hail at upon xs praying i` | `T` | element at index `i` (zero-based) |
| `hail first upon xs` | `grace of T` | first element or `absent` |
| `hail last upon xs` | `grace of T` | last element or `absent` |
| `hail contains upon xs praying v` | `dogma` | whether `v` is in the legion |
| `hail index_of upon xs praying v` | `grace of atom` | position of `v` or `absent` |
| `hail reverse upon xs` | `legion of T` | new reversed legion |
| `hail push upon xs praying v` | `legion of T` | new legion with `v` appended |
| `hail slice upon xs praying start and end` | `legion of T` | sub-legion `[start, end)` |
| `hail concat upon xs praying ys` | `legion of T` | new legion with `ys` appended |

> **Important:** all methods that return `legion of T` (`push`, `slice`, `concat`, `reverse`) **do not mutate** the existing legion — they return a new one. Reassign if you need to keep the result:
> ```holy
> xs become hail push upon xs praying 99
> xs become hail reverse upon xs
> ```

---

## Practical examples

### Iterating a legion

```holy
let there xs of legion of atom be hail legion praying 10, 20 and 30
let there i  of atom be 0

litany for i lesser than hail length upon xs
    hail proclaim praying hail word_of praying hail at upon xs praying i
    i become i plus 1
```

### Building a legion dynamically

```holy
let there be result of legion of atom
let there i of atom be 1

litany for i no greater than 5
    result become hail push upon result praying i times i   -- squares: 1, 4, 9, 16, 25
    i become i plus 1

hail proclaim praying hail word_of praying result
```

### Slicing and concatenating

```holy
let there xs   of legion of atom be hail legion praying 1, 2, 3, 4 and 5
let there head of legion of atom be hail slice upon xs praying 0 and 2   -- [1, 2]
let there tail of legion of atom be hail slice upon xs praying 2 and 5   -- [3, 4, 5]
let there all  of legion of atom be hail concat upon head praying tail    -- [1, 2, 3, 4, 5]
```

---

## Generic `legion`

You can declare scriptures and salms that work with `legion of T`:

```holy
salm first of T receiving xs of legion of T reveals verdict of T and word
    whether hail is_empty upon xs
        reveal manifest condemned of verdict of T and word praying "legion is empty"
    reveal manifest righteous of verdict of T and word praying hail at upon xs praying 0
```

`at` throws `IndexOutOfBounds` for out-of-range indices. `slice` with `start > end` or `end > length` also throws `IndexOutOfBounds`.
