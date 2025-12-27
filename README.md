# Lipona

**toki pona pi ilo nanpa**

Lipona li toki pi ilo nanpa. ona li kepeken toki pona. sina ken pali e ijo mute kepeken ona.

## nasin open

```bash
# pali e ilo
cargo build

# kepeken ilo
cargo run -- lipu.lipo

# kepeken toki
cargo run -- -e 'toki("pona")'
```

## nasin toki

### nanpa en sitelen

```
// nanpa
x jo 42
y jo 3.14

// sitelen
nimi jo "jan Lipona"

// sitelen pi ijo insa
toki("toki, {nimi}!")
```

### ijo lon en ijo ala

```
a jo lon    // lon (true)
b jo ala    // ala (false/null)
```

### pali nanpa

```
// + - * /
x jo 10 + 5
y jo x * 2
```

### lukin sama

```
a suli b        // a > b
a lili b        // a < b
a suli_sama b   // a >= b
a lili_sama b   // a <= b
a sama b        // a == b
```

### la (ken)

```
x suli 10 la open
    toki("suli")
pini taso open
    toki("lili")
pini
```

### wile (sin sin sin)

```
i jo 0
wile i lili 5 la open
    toki(i)
    i jo i + 1
pini
```

### ilo (pali)

```
// pali sin
ilo sum (a, b) open
    pana a + b
pini

// kepeken pali
x jo sum(10, 20)
toki(x)  // 30
```

## ilo insa (stdlib)

| ilo | nasin |
|-----|-------|
| `toki(x)` | sitelen tawa jan |
| `nanpa_sin(s)` | sitelen tawa nanpa |
| `sitelen_len(s)` | suli pi sitelen |
| `kulupu_sin(...)` | pali e kulupu |
| `kulupu_len(k)` | suli pi kulupu |
| `kulupu_ken(k, i)` | kama jo tan kulupu |
| `nasin_sin()` | pali e nasin |
| `nasin_lon(n, k, v)` | pana tawa nasin |
| `nasin_ken(n, k)` | kama jo tan nasin |

## sona ante

- lipu li `.lipo`
- `//` li toki pi jan ala (comment)
- nimi li ASCII taso

---

**pali kepeken Rust. pona tawa sina!**
