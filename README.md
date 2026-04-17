# Trygve Bjerkreim

> *ei tue verd for full* — a world worth everything

Trygve Bjerkreim is a programming language whose syntax consists entirely of phrases from
the poetry and hymns of **Trygve Bjerkrheim** (1904–2001), Norwegian poet, hymn
writer, and editor.

The interpreter is called **tbv** (Trygve Bjerkrheim's Verse).

---

## Running programs

```
./tbv program.tb        # run a file
./tbv --repl                # interactive REPL
./tbv --version             # show version
```

Files use the `.tb` extension. Source must be UTF-8.

---

## Comments

```
– – dette er ein kommentar – –
```

En-dash pairs (`–`) mark a comment to end of line, exactly as they appear in
his poetry.

---

## Types

| Type | Example | Display |
|------|---------|---------|
| Integer | `42` | `42` |
| Float | `3.14` | `3.14` |
| String | `«Aldri var landet so fagert som no»` | as-is |
| Boolean true | `ja` | `ja` |
| Boolean false | `nei` | `nei` |
| Null | `tome hender` | `tome hender` |
| List | `[1, 2, 3]` | `[1, 2, 3]` |

Strings use Norwegian guillemets `«` and `»`.

---

## Variables

**Declare / assign** — *"To give is to sow [value] into [name]"*

```
Å gi er å så 42 til svaret
Å gi er å så «hei» til helsing
Å gi er å så [1, 2, 3] til liste
```

**Reassign** — *"[name] receives [value]"*

```
svaret tek imot svaret og 1
```

Variable names are single words (lowercase recommended). They cannot start with
a capitalised keyword.

---

## Output

*"Our song shall rise"* — prints a value followed by newline.

```
Vår song skal stiga opp: «Aldri var landet so fagert som no»
Vår song skal stiga opp: 6 gongar 7
```

---

## Input

*"Come with your [name]"* — reads one line from stdin into a variable.

```
Kom med din svar
```

The value is always a string. Use `heiltal(svar)` to convert.

---

## Arithmetic

| Operation | Syntax | Example |
|-----------|--------|---------|
| Addition / string concat | `a og b` | `3 og 4` → `7` |
| Subtraction | `a utan b` | `10 utan 3` → `7` |
| Multiplication | `a gongar b` | `6 gongar 7` → `42` |
| Integer division | `a delt på b` | `10 delt på 3` → `3` |
| Modulo | `resten av a delt på b` | `resten av 10 delt på 3` → `1` |

String concat: if either operand is a string, `og` concatenates.

---

## Comparison

| Operation | Syntax |
|-----------|--------|
| Equal | `a er b` |
| Not equal | `a er ikkje b` |
| Less than | `a er mindre enn b` |
| Greater than | `a er større enn b` |

Returns `ja` (true) or `nei` (false).

---

## Logical NOT

```
ikkje ja          – – → nei – –
ikkje a er b      – – NOT (a == b) — use parentheses for clarity – –
```

---

## Conditionals

*"You can't get around [condition]"* — if the condition holds, the body runs.

```
Du kjem ikkje utanom svar er 42:
    Vår song skal stiga opp: «Rett!»
Det er nok.
```

With else — *"but if not"*:

```
Du kjem ikkje utanom tal er større enn 0:
    Vår song skal stiga opp: «positivt»
Men om ikkje:
    Vår song skal stiga opp: «negativt eller null»
Det er nok.
```

Every `Du kjem ikkje utanom` block ends with `Det er nok.`

---

## While loop

*"Just one day, one moment at a time, while [condition]"*

```
Å gi er å så 1 til i
Blott ein dag, eit øyeblikk om gongen, medan i er mindre enn 11:
    Vår song skal stiga opp: i
    i tek imot i og 1
Det er nok.
```

---

## For-each loop

*"Each day is precious [var] in [iterable]"*

```
Kvar dag er dyr song i songar:
    Vår song skal stiga opp: song
Det er nok.
```

Works on lists and strings.

---

## Count loop

*"Peak behind peaks [n] times"* — from the Romsdal poem, where peaks nest behind
peaks: `Topp attom toppar, tind attom tind`.

```
Topp attom toppar 5 gongar:
    Vår song skal stiga opp: «hei»
Det er nok.
```

With a loop variable — *"as [var]"* (0-indexed):

```
Topp attom toppar 5 som i gongar:
    Vår song skal stiga opp: i
Det er nok.
```

---

## Infinite loop

*"Eternal in the kingdom of light"*

```
Evig i lysets rike:
    – – this runs forever – –
    – – use Du kjem ikkje utanom … to break – –
Det er nok.
```

To break out: set a flag and use a while loop instead, or restructure with
`Blott ein dag`.

---

## Functions

**Define** — *"God has a plan for [name]"*

```
Gud har ein plan med kvadrat(n):
    Takk for sangen: n gongar n
Det er nok.
```

**Return** — *"Thanks for the song"* (the song has been sung; it rises back up)

```
Takk for sangen: n gongar n
```

**Call as statement** — *"Come along to [name] with [args]"*

```
Bli med til kvadrat med 5
```

**Call as expression** (same syntax, used inside an expression):

```
Vår song skal stiga opp: Bli med til kvadrat med 7
Å gi er å så Bli med til kvadrat med n til resultat
```

**Call with parentheses** (alternative syntax for built-ins and functions):

```
Vår song skal stiga opp: kvadrat(7)
```

Functions without `Takk for sangen` return `tome hender` (null).

---

## Error handling

*"Try to get done what you can / Don't be afraid"*

```
Prøv å få gjort det du kan:
    – – risky code – –
    Å gi er å så heiltal(«ikkje eit tal») til n
Ver ikkje redd:
    Vår song skal stiga opp: «Noko gjekk gale: » og feilen
Det er nok.
```

The variable `feilen` contains the error message inside the catch block.

---

## Built-in functions

| Name | Arguments | Returns |
|------|-----------|---------|
| `lengd(v)` | list or string | integer length |
| `heiltal(v)` | any | integer |
| `desimaltal(v)` | any | float |
| `tekst(v)` | any | string |
| `legg_til(liste, x)` | list, value | new list with x appended |
| `del_frå(liste, i)` | list, index | new list with element i removed |
| `del_opp(s, sep)` | string, separator | list of substrings |
| `sett_saman(liste, sep)` | list, separator | joined string |
| `sorter(liste)` | list | sorted list |
| `kvart_tal(n)` | integer | list `[0, 1, …, n-1]` |

---

## Operator precedence (high to low)

1. `name(args)` call, `liste[i]` index
2. `resten av … delt på …` modulo, `ikkje …` logical NOT
3. `gongar`, `delt på` — multiply, divide
4. `og`, `utan` — add/concat, subtract
5. `er`, `er ikkje`, `er mindre enn`, `er større enn` — comparison

Use parentheses `(…)` to override precedence.

---

## Notes

- **`og` is addition**, not boolean AND. For compound conditions, use nested `Du kjem ikkje utanom` blocks.
- Function call arguments are greedy. When a function call is part of a larger expression, store intermediate results in variables:
  ```
  Å gi er å så Bli med til f med a til x
  Å gi er å så Bli med til g med b til y
  Vår song skal stiga opp: x og y
  ```
  Or use parentheses: `(Bli med til f med a) og (Bli med til g med b)`
- Variable names should be single lowercase Norwegian words. Underscores are allowed (`er_primtal`).

---

## Examples

| File | Description |
|------|-------------|
| `examples/hei_verd.tb` | Hello, world |
| `examples/fizzbuzz.tb` | FizzBuzz 1–100 |
| `examples/fibonacci.tb` | Fibonacci sequence |
| `examples/gjeting.tb` | Number guessing game |
| `examples/lister.tb` | Lists and iteration |
| `examples/funksjonar.tb` | Functions, factorial, primality |

---

## The poet

Trygve Bjerkrheim (1904–2001) was born in Bjerkreim in Rogaland, Norway, and
grew up in Høland in Akershus. He spent most of his life near Fjellhaug Schools
in Oslo, working as a teacher, editor of the mission magazine *Utsyn*, and
prolific author of poetry, hymns, and prose. His collected works fill five
volumes, published posthumously by Lunde Forlag (2001–2002). Many of his hymns
remain in active use in Norwegian churches today.

The language is named after its author, Trygve Bjerkrheim.
