# Disambiguation with `thus` and `after`

Holy uses a word-based syntax, so nesting can become ambiguous when an inner type, call, or grouped expression is followed by tokens that belong to the outer context.

In Holy lists, the final separator may be written as `and`: `a and b`, `a, b and c`, `Pair of atom and word`.

The two tools for disambiguation are:

- `thus`: closes one open inner context
- `after … thus`: groups an expression explicitly

This page focuses on one question:

> When does the parser need help to know where the inner construct ends and the outer one resumes?

---

## The rule of thumb

Use `thus` when:

- an inner generic type is followed by a separator that belongs to an outer type
- an inner `hail` or `manifest` call is followed by `,` or `and` that belongs to an outer call
- an `after` group must end before the outer expression continues

Each `thus` closes exactly one level.

---

## 1. Disambiguating expression precedence with `after`

By default, Holy uses operator precedence.

```holy
let there x of atom be 2 plus 3 times 4
```

This is read as:

```holy
2 plus (3 times 4)
```

If you want the parser to read a different grouping, open a grouped sub-expression with `after` and close it with `thus`:

```holy
let there x of atom be after 2 plus 3 thus times 4
```

This is read as:

```holy
(2 plus 3) times 4
```

So `after … thus` is the disambiguation tool for expression grouping.

Another example:

```holy
let there y of atom be 5 plus after 10 minus 3 thus times 2
```

This is read as:

```holy
5 plus ((10 minus 3) times 2)
```

Without `after … thus`, the parser would not treat `10 minus 3` as one protected inner expression.

---

## 2. Disambiguating nested calls

When a `hail` call appears inside another `hail` call, the parser must know where the inner argument list ends. This happens whether the outer separator is `,` or `and`.

### No ambiguity: inner call is the last outer argument

```holy
hail proclaim praying hail word_of praying 42
```

This is unambiguous because the inner call reaches the end of the outer argument list.

### Ambiguous: inner call is followed by more outer arguments

```holy
hail add praying hail double praying 3 and 2
```

At a glance, a reader may intend this to mean:

```text
add(double(3), 2)
```

But that is **not** how the parser reads it.

Because the inner call is still open, `and 2` is consumed by `double`, so the parser reads it like this:

```text
add(double(3, 2))
```

In other words:

- intended outer reading: second argument of `add`
- actual parse: second argument of `double`

To close the inner call before continuing the outer one, use `thus`:

```holy
hail add praying hail double praying 3 thus and 2
```

This is read as:

```text
add(double(3), 2)
```

### More than one nested call

```holy
hail a praying hail b praying hail c praying 1 thus thus and 2
```

Read it step by step:

1. first `thus` closes `c(...)`
2. second `thus` closes `b(...)`
3. `and 2` now belongs to `a(...)`

So the full reading is:

```holy
a(b(c(1)), 2)
```

---

## 3. The same rule for `manifest`

`manifest` uses the same nesting rule as `hail`.

### Unambiguous

```holy
manifest Box praying manifest Point praying 1 and 2
```

If the inner manifestation is the last outer argument, no extra `thus` is needed.

### Ambiguous

```holy
manifest Box praying manifest Point praying 1 and 2 and 99
```

That keeps the parser inside the inner `manifest Point ...`.

To close the inner manifestation and continue the outer one:

```holy
manifest Box praying manifest Point praying 1 and 2 thus and 99
```

This is read as:

```holy
Box(Point(1, 2), 99)
```

---

## 4. Disambiguating nested generic types

Generic types use `of`. Ambiguity appears when an inner generic type is followed by a separator that belongs to an outer type.

### No ambiguity

```holy
grace of atom
verdict of atom, word
```

These are simple because `atom` and `word` are not themselves generic.

### Ambiguous

```holy
verdict of Stack of atom, word
```

After reading `Stack of atom`, the parser can continue greedily and treat `word` as belonging to the inner type context.

To close the inner generic type, use `thus`:

```holy
verdict of Stack of atom thus and word
```

This is read as:

```holy
verdict<Stack<atom>, word>
```

### Another generic nesting example

```holy
Pair of grace of atom thus, word
```

This is read as:

```holy
Pair<grace<atom>, word>
```

Again, `thus` closes the inner generic `grace of atom` before the outer separator.

---

## 5. The same ambiguity inside declarations

This is not only for values. The same disambiguation rule applies anywhere a type is written.

### Parameter types

```holy
salm show receiving value of verdict of Stack of atom thus and word reveals word
    reveal hail word_of praying value
```

Here, `thus` tells the parser:

- `Stack of atom` is complete
- the outer separator belongs to `verdict`
- it does not belong to the outer parameter list

### Return types

```holy
salm load reveals verdict of grace of atom thus and word
    reveal manifest condemned of verdict of grace of atom thus and word praying "not implemented"
```

Here the return type is:

```holy
verdict<grace<atom>, word>
```

---

## 6. Type disambiguation and call disambiguation together

This is the case that usually needs the most care.

```holy
let there answer of verdict of Stack of atom thus and word be hail build praying hail choose praying 1 thus and 2
```

This line uses two different disambiguations:

- `thus` in `Stack of atom thus and word` closes the inner generic type before the outer separator of `verdict`
- `thus` in `hail choose praying 1 thus and 2` closes the inner call before the outer `and 2`

They solve different ambiguities in the same line.

---

## 7. `after` inside a nested call

Sometimes you need both expression grouping and call disambiguation.

```holy
hail sum3 praying 1, hail double praying after 3 plus 4 thus thus and 9
```

Read the `thus` tokens in order:

1. the first `thus` closes `after 3 plus 4`
2. the second `thus` closes `hail double praying ...`
3. `and 9` now belongs to the outer `hail sum3 ...`

So this is read as:

```holy
sum3(1, double((3 plus 4)), 9)
```

Without the first `thus`, the grouped expression would stay open.  
Without the second `thus`, the parser would keep reading as if `and 9` still belonged to `double`.

---

## 8. Wrong vs correct

### Nested call before another outer argument

Wrong:

```holy
hail add praying hail double praying 3 and 2
```

Correct:

```holy
hail add praying hail double praying 3 thus and 2
```

### Nested generic before an outer separator

Wrong:

```holy
verdict of Stack of atom, word
```

Correct:

```holy
verdict of Stack of atom thus and word
```

### Grouping inside a larger expression

Wrong:

```holy
2 plus 3 thus times 4
```

Correct:

```holy
after 2 plus 3 thus times 4
```

`thus` does not open a context.  
It only closes one that was opened by `after`, `praying`, or a nested generic `of`.

---

## 9. Mental model

The parser opens contexts like this:

- `after` opens a grouped expression
- `praying` opens a call argument list
- `of` may open a nested generic type

Then:

- each `thus` closes one currently open context

So when you see:

```holy
hail outer praying hail inner praying after 1 plus 2 thus thus and 9
```

read it as:

1. open grouping with `after`
2. first `thus` closes the grouped expression
3. second `thus` closes `inner(...)`
4. `and 9` resumes the outer `outer(...)` call

---

## See also

- [Types & Variables](types.md#expression-grouping--after--thus)
- [Salms](salms.md#nested-calls)
- [Generics](generics.md#thus--disambiguation)
