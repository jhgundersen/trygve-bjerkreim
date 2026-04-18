# Trygve Bjerkreim

> *ei tue verd for full* — a world worth everything

Trygve Bjerkreim is a programming language whose syntax consists entirely of phrases from
the poetry and hymns of **Trygve Bjerkrheim** (1904–2001), Norwegian poet, hymn
writer, and editor.

The interpreter is called **tbv** (Trygve Bjerkrheim's Verse).

---

## Running programs

```
./tbv program.tb     # run a file
./tbv --repl         # interactive REPL
./tbv --version      # show version
```

Files use the `.tb` extension. Source must be UTF-8.

---

## Comments

```
– – dette er ein kommentar – –
```

En-dash pairs (`–`) mark a comment to end of line, exactly as they appear in his poetry.

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

**Declare** — *"let [name] be [value]"*

```
lat svaret vera 42
lat helsing vera «hei»
lat liste vera [1, 2, 3]
```

**Reassign** — *"[name] receives [value]"*

```
svaret tek imot svaret og 1
```

---

## Output

*"Sing out"* — prints a value followed by newline.

```
Syng ut: «Aldri var landet so fagert som no»
Syng ut: 6 gongar 7
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

*"You can't get around [condition]"*

```
Du kjem ikkje utanom svar er 42:
    Syng ut: «Rett!»
Det er nok.
```

With else — *"but if not"*:

```
Du kjem ikkje utanom tal er større enn 0:
    Syng ut: «positivt»
Men om ikkje:
    Syng ut: «negativt eller null»
Det er nok.
```

With else-if — *"but if [condition]"*:

```
Du kjem ikkje utanom x er 1:
    Syng ut: «ein»
Men om x er 2:
    Syng ut: «to»
Men om x er 3:
    Syng ut: «tre»
Men om ikkje:
    Syng ut: «anna»
Det er nok.
```

Every `Du kjem ikkje utanom` block ends with a single `Det er nok.`

---

## While loop

*"One moment at a time, while [condition]"*

```
lat i vera 1
Eit øyeblikk om gangen, medan i er mindre enn 11:
    Syng ut: i
    i tek imot i og 1
Det er nok.
```

---

## For-each loop

*"each [var] in [iterable]"*

```
kvar song i songar:
    Syng ut: song
Det er nok.
```

Works on lists and strings.

---

## Count loop

*"Peak behind peaks [n] times"* — from `Topp attom toppar, tind attom tind`.

```
Topp attom toppar 5 gongar:
    Syng ut: «hei»
Det er nok.
```

With a loop variable — *"as [var]"* (0-indexed):

```
Topp attom toppar 5 som i gongar:
    Syng ut: i
Det er nok.
```

---

## Infinite loop

*"Eternal in the kingdom of light"*

```
Evig i lysets rike:
    – – runs forever – –
Det er nok.
```

---

## Break and continue

**Break** — *"the shuttle stops still"* (from `og skyttelen stansar stilt`)

```
stansar stilt.
```

**Continue** — *"once again"* (from `Atter ein gong ser eg`)

```
atter ein gong.
```

---

## Functions

**Define** — *"God has a plan for [name]"*

```
Gud har ein plan med kvadrat(n):
    Takk at du tok mine byrder: n gongar n
Det er nok.
```

**Return** — *"Thanks that you took my burdens"*

```
Takk at du tok mine byrder: n gongar n
```

**Call as statement** — *"Come along to [name] with [args]"*

```
Bli med til kvadrat med 5
```

**Call as expression** (same syntax, inside an expression):

```
Syng ut: Bli med til kvadrat med 7
lat resultat vera Bli med til kvadrat med n
```

**Call with parentheses** (alternative, works for both builtins and functions):

```
Syng ut: kvadrat(7)
```

Functions without `Takk at du tok mine byrder` return `tome hender` (null).

---

## Error handling

*"Try to get done what you can / Don't be afraid"*

```
Prøv å få gjort det du kan:
    lat n vera heiltal(«ikkje eit tal»)
Ver ikkje redd:
    Syng ut: «Noko gjekk gale: » og feilen
Det er nok.
```

The variable `feilen` contains the error message inside the catch block.

---

## Raise error

*"Cry out"* — from *"Rop ut til kvart eit folkeslag"* (song 760). Raises a runtime error that can be caught with `Ver ikkje redd:`.

```
Rop ut: «noko gjekk gale»
```

---

## Assert

*"Set guard"* — from *"Set då vakt ved hjartans port"* (song 527). Halts with an error if the condition is false.

```
Set vakt: lengd(liste) er større enn 0
Set vakt: n er ikkje 0
```

---

## Sleep

*"Rest a moment"* — pauses execution for N seconds (accepts decimals).

```
Kvil eit augneblink: 1
Kvil eit augneblink: 0.5
```

---

## Web server

*"Listen at port [n]"* — starts an HTTP server. Sets `metode`, `vegen`, and `kropp`
for each incoming request. *"Answer with [expr]"* sends the response.

```
Lytt ved port 8080:
    Du kjem ikkje utanom vegen er «/hei»:
        Svar med: «Hei, verd!»
    Det er nok.
    Svar med: «404 – ikkje funne: » og vegen
Det er nok.
```

The server runs until the process is killed. Each request gets a fresh scope.
`Svar med:` may be called once; the last value wins if called multiple times.

---

## Built-in functions

| Name | Arguments | Returns |
|------|-----------|---------|
| `lengd(v)` | list or string | integer length |
| `heiltal(v)` | any | integer |
| `desimaltal(v)` | any | float |
| `tekst(v)` | any | string |
| `legg til(liste, x)` | list, value | new list with x appended |
| `del frå(liste, i)` | list, index | new list with element i removed |
| `del opp(s, sep)` | string, separator | list of substrings |
| `sett saman(liste, sep)` | list, separator | joined string |
| `sorter(liste)` | list | sorted list |
| `kvart tal(n)` | integer | list `[0, 1, …, n-1]` |

Two-word builtins can also be called with parentheses: `legg til(liste, x)`.

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

- **`og` is addition**, not boolean AND. For compound conditions, nest `Du kjem ikkje utanom` blocks.
- Function call arguments are greedy. Store intermediate results in variables or use parentheses:
  ```
  lat x vera Bli med til f med a
  lat y vera Bli med til g med b
  Syng ut: x og y
  ```

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
| `examples/webtenar.tb` | Simple HTTP web server |

---

## The poet

Trygve Bjerkrheim (1904–2001) was born in Bjerkreim in Rogaland, Norway, and
grew up in Høland in Akershus. He spent most of his life near Fjellhaug Schools
in Oslo, working as a teacher, editor of the mission magazine *Utsyn*, and
prolific author of poetry, hymns, and prose. His collected works fill five
volumes, published posthumously by Lunde Forlag (2001–2002). Many of his hymns
remain in active use in Norwegian churches today.

The language is named after its author, Trygve Bjerkrheim.
