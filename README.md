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
cargo run -- -e 'toki e ("pona")'
```

## nasin toki

### nanpa en sitelen

```
// nanpa
x li jo e 42
y li jo e 3.14

// sitelen
nimi li jo e "jan Lipona"

// sitelen pi ijo insa
toki e ("toki, {nimi}!")
```

### ijo lon en ijo ala

```
a li jo e lon    // lon (true)
b li jo e ala    // ala (false/null)
```

### pali nanpa

```
// + - * /
x li jo e 10 + 5
y li jo e x * 2
```

### lukin sama

```
a li suli e b        // a > b
a li lili e b        // a < b
a li suli_sama e b   // a >= b
a li lili_sama e b   // a <= b
a li sama e b        // a == b
```

### la (ken)

```
x li suli e 10 la open
    toki e ("suli")
pini taso open
    toki e ("lili")
pini
```

### wile (sin sin sin)

```
i li jo e 0
wile i li lili e 5 la open
    toki e (i)
    i li jo e i + 1
pini
```

### ilo (pali)

```
// pali sin
ilo sum li pali e (a, b) la open
    pana e a + b
pini

// kepeken pali
x li jo e sum e (10, 20)
toki e (x)  // 30
```

## ilo insa (stdlib)

| ilo | nasin |
|-----|-------|
| `toki e (x)` | sitelen tawa jan |
| `nanpa_sin e (s)` | sitelen tawa nanpa |
| `sitelen_len e (s)` | suli pi sitelen |
| `kulupu_sin e (...)` | pali e kulupu |
| `kulupu_len e (k)` | suli pi kulupu |
| `kulupu_ken e (k, i)` | kama jo tan kulupu |
| `nasin_sin e ()` | pali e nasin |
| `nasin_lon e (n, k, v)` | pana tawa nasin |
| `nasin_ken e (n, k)` | kama jo tan nasin |

## sona ante

- lipu li `.lipo`
- `//` li toki pi jan ala (comment)
- nimi li ASCII taso

---

**pali kepeken Rust. pona tawa sina!**
