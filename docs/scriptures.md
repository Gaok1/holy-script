# Scriptures

Scriptures are pure data structures — named collections of typed fields with no behaviour of their own. Behaviour is added through [method salms](#method-salms).

---

## Declaration

```holy
scripture Point
    x of atom
    y of atom

scripture Person
    name of word
    age  of atom
```

- At least one field is required.
- Fields are declared in order; that order is used when instantiating.
- Field names must be unique within the scripture.

---

## Instantiation — `manifest`

```holy
let there p of Point be manifest Point praying 3, 4
let there u of Person be manifest Person praying "Gabriel", 30
```

Arguments are passed **in field declaration order**, separated by `,`.  
As with other Holy lists, the final separator may use `and` instead of `,`:

```holy
let there p of Point be manifest Point praying 3 and 4
```

---

## Field access — `from`

```holy
let there px of atom be x from p
let there nm of word be name from u
```

`from` reads a single field by name. It does **not** mutate the value.

If a field contains another scripture, `from` can be chained from left to right:

```holy
let there nested of word be nestedField from field from p
```

This reads `p.field.nestedField`.

## Value semantics

Scriptures behave as whole values, not as mutable containers with assignable inner fields.

- You can read fields with `from`.
- You cannot assign to a field directly.
- To "change" a scripture, create a new manifestation and reassign the variable that holds it.

```holy
scripture Person
    name of word
    age of atom

let there p of Person be manifest Person praying "Gabriel", 30

-- not allowed:
-- age from p become 31

-- allowed: replace the whole value
p become manifest Person praying name from p, 31
```

The same rule applies to nested scriptures: inner values are not updated in place. Build a new inner scripture, then a new outer scripture, and reassign the outer variable.

### Chain access

Fields can be chained when a field is itself a scripture:

```holy
scripture Address
    city of word

scripture Employee
    person of Person
    address of Address

let there city of word be city from address from emp
-- reads emp.address.city
```

### Inside a method salm — `its`

Within a `salm … upon SomeType` body, `its` refers to the instance the method was called on:

```holy
salm fullName upon Person reveals word
    reveal name from its plus " (age " plus hail word_of praying age from its plus ")"
```

---

## Method salms

A method salm is bound to a scripture type via `upon`. It is called on an instance of that type.

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

- Inside the body, `its` has the type of the bound scripture.
- `its` is read-only; you cannot reassign it.
- Method salms can also have type parameters (see [Generics](generics.md)).

---

## Generic scriptures

Scriptures can declare type parameters with `of`. The final type parameter separator may also be `and`:

```holy
scripture Pair of A and B
    first  of A
    second of B

scripture Box of T
    value of T
```

Type parameters are abstract names resolved at the call site. They appear in field type annotations and are passed explicitly when instantiating.

```holy
let there b of Box of atom be manifest Box praying 42
let there p of Pair of atom and word be manifest Pair praying 1 and "x"
```

When a generic type appears before a comma that separates other arguments or type args, use `thus` to close it first:

```holy
-- Box<Stack<T>> — thus closes Stack<T> before the comma
let there x of Box of Stack of T thus be manifest Box praying s
```

See [Generics](generics.md) for the generic-type rule and [Disambiguation with `thus` and `after`](nesting.md) for the broader nesting/disambiguation model.

---

## Constructor convention

By convention, a salm with the same name as a scripture acts as a constructor:

```holy
scripture Point
    x of atom
    y of atom

salm Point receiving x of atom, y of atom reveals Point
    reveal manifest Point praying x, y

-- usage is cleaner
let there p of Point be hail Point praying 3, 4
```

---

## Default values

When declared without a value (`let there be`), a scripture variable is initialised to `void`. Accessing its fields before assigning a proper value will produce a runtime error.

```holy
let there be p of Point         -- p = void internally
p become manifest Point praying 0, 0   -- safe now
let there px of atom be x from p
```
