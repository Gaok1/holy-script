# Modules

Holy supports modules through the `testament` declaration. Imports appear at the top of the file, before any declarations or statements. Each module corresponds to a `.holy` file resolved relative to the importing file.

---

## Importing a module — `testament`

```holy
testament MathUtils
```

Imports all public symbols (scriptures, covenants, salms, sins) from `MathUtils.holy` into the current program's scope.

The file `MathUtils.holy` must be in the same directory as the importing file:

```
project/
  main.holy
  MathUtils.holy   ← resolved automatically
```

---

## Subdirectory imports — `from`

Use one or more `from` clauses to navigate into subdirectories:

```holy
testament utils from helpers
testament models from db from schema
```

Each `from` adds one level to the path. The rules:

| Declaration | Resolved path |
|-------------|---------------|
| `testament utils from helpers` | `helpers/utils.holy` |
| `testament models from db from schema` | `db/schema/models.holy` |

All paths are relative to the directory of the importing file.

```
project/
  main.holy
  helpers/
    utils.holy        ← testament utils from helpers
  db/
    schema/
      models.holy     ← testament models from db from schema
```

---

## Selective import — `revealing`

```holy
testament MathUtils revealing square, cube
testament Collections revealing Stack and Queue
testament utils from helpers revealing trim, split
```

Only the listed symbols are imported. The rest remain inaccessible. The final separator may be `and`. `revealing` always comes after any `from` clauses.

```holy
-- MathUtils.holy exports square, cube and is_even
-- but only square was imported:
testament MathUtils revealing square

let there s of atom be hail square praying 5    -- ok
let there c of atom be hail cube praying 3      -- UndefinedSalm: cube was not imported
```

---

## Standard library — `arithmos`

Holy ships with a built-in standard library. If a `testament` with no `from` path is not found as a local file, the interpreter checks the stdlib automatically.

The only current stdlib module is **`arithmos`** (from Αριθμοί, the Greek Bible book of Numbers):

```holy
testament arithmos
```

Provides mathematical constants and angle converters:

| Salm | Returns | Value |
|------|---------|-------|
| `pi` | `fractional` | 3.141592653589793 |
| `euler` | `fractional` | 2.718281828459045 (*e*) |
| `tau` | `fractional` | 6.283185307179586 |
| `to_degrees(rad)` | `fractional` | Converts radians to degrees |
| `to_radians(deg)` | `fractional` | Converts degrees to radians |

```holy
testament arithmos

let there p of fractional be hail pi
let there d of fractional be hail to_degrees praying p
hail proclaim praying hail word_of praying d      -- 180.0

let there half_turn of fractional be hail tau over 2
hail proclaim praying hail word_of praying hail sine praying half_turn   -- ~1.0

amen
```

`revealing` works normally with stdlib modules:

```holy
testament arithmos revealing pi, to_degrees
```

---

## Multiple imports

```holy
testament MathUtils
testament arithmos
testament Collections revealing Stack
testament utils from helpers revealing trim, split
```

Imports are processed in declaration order. Already-loaded modules are silently skipped (no re-importing).

---

## Module file structure

A module file is a normal `.holy` file — it can contain scriptures, covenants, sins, salms, and even imports of other modules. Executable top-level statements in a module file **are not executed** — only declarations are imported.

```holy
-- MathUtils.holy
salm square receiving n of atom reveals atom
    reveal n times n

salm cube receiving n of atom reveals atom
    reveal n times n times n

salm is_even receiving n of atom reveals dogma
    whether n remainder 2 is 0
        reveal blessed
    reveal forsaken

amen
```

```holy
-- main.holy
testament MathUtils

hail proclaim praying hail word_of praying hail square praying 7   -- 49

amen
```

---

## Modules importing modules

A module can import other modules. The interpreter resolves dependencies recursively, respecting order and avoiding re-imports:

```holy
-- Vectors.holy
testament arithmos

scripture Vector2D
    x of fractional
    y of fractional

salm magnitude upon Vector2D reveals fractional
    reveal hail sqrt praying after x from its times x from its plus y from its times y from its

amen
```

---

## Full syntax

```holy
testament ModuleName
testament ModuleName from subdir
testament ModuleName from dir1 from dir2
testament ModuleName revealing Symbol1, Symbol2 and Symbol3
testament ModuleName from subdir revealing Symbol1 and Symbol2
```

- `ModuleName` is an identifier — it must match the filename (without `.holy`).
- `from ident` clauses are optional and can be chained; each adds one subdirectory level.
- `revealing` is optional and must come after all `from` clauses.
- All `testament` declarations must come before any `scripture`, `sin`, `covenant`, `salm`, or statement.

---

## Current limitations

- Circular imports (`A` imports `B` which imports `A`) are not detected — they are silently avoided by the already-loaded set, but symbols declared in `A` will not be visible to `B`.
