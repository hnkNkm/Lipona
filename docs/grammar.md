# Lipona Grammar Reference

Lipona は Toki Pona の文法構造に基づいたプログラミング言語です。

## 基本構文

### 変数代入

```
<識別子> jo <式>
```

例:
```
x jo 42
name jo "jan Lipona"
result jo add(10, 20)
```

### 関数呼び出し

```
<関数名>(<引数>, ...)
```

例:
```
toki("pona")
sum(10, 20)
kulupu_ken(arr, 0)
```

### 関数定義

```
ilo <関数名> (<引数>, ...) open
    <文>...
    pana <戻り値>
pini
```

例:
```
ilo double (n) open
    pana n * 2
pini

ilo add (a, b) open
    pana a + b
pini
```

### 条件分岐 (if/else)

```
<条件> la open
    <文>...
pini taso open
    <文>...
pini
```

`taso open ... pini` (else節) は省略可能。

例:
```
x suli 10 la open
    toki("suli")
pini taso open
    toki("lili")
pini
```

### ループ (while)

```
wile <条件> la open
    <文>...
pini
```

例:
```
i jo 0
wile i lili 10 la open
    toki(i)
    i jo i + 1
pini
```

### 戻り値 (return)

```
pana <式>
```

## データ型

| 型 | 説明 | 例 |
|----|------|-----|
| nanpa | 数値 (64bit float) | `42`, `3.14`, `-10` |
| sitelen | 文字列 | `"pona"`, `"Hello, {name}!"` |
| lon | 真 (true) | `lon` |
| ala | 偽/null | `ala` |
| kulupu | リスト | `kulupu_sin(1, 2, 3)` |
| nasin | マップ | `nasin_sin()` |
| ilo | 関数 | `ilo f () open ... pini` |

## 演算子

### 算術演算子

| 演算子 | 説明 | 例 |
|--------|------|-----|
| `+` | 加算 | `10 + 5` → `15` |
| `-` | 減算 | `10 - 5` → `5` |
| `*` | 乗算 | `10 * 5` → `50` |
| `/` | 除算 | `10 / 4` → `2.5` |

文字列の連結にも `+` を使用:
```
"toki " + "pona"  // → "toki pona"
```

### 比較演算子

| 構文 | 意味 | 例 |
|------|------|-----|
| `a suli b` | a > b | `5 suli 3` → `lon` |
| `a lili b` | a < b | `3 lili 5` → `lon` |
| `a suli_sama b` | a >= b | `5 suli_sama 5` → `lon` |
| `a lili_sama b` | a <= b | `5 lili_sama 5` → `lon` |
| `a sama b` | a == b | `5 sama 5` → `lon` |

比較結果は `lon` (真) または `ala` (偽) を返す。

## 文字列

### 基本文字列

```
"pona"
"Hello, World!"
```

### 文字列補間

`{式}` で変数や式を埋め込める:

```
name jo "jan"
toki("toki, {name}!")     // → "toki, jan!"

x jo 10
toki("x * 2 = {x * 2}")   // → "x * 2 = 20"
```

**注意**: 補間内でスペースを含む式は変数に代入してから使用:
```
// OK
result jo a + b
toki("sum = {result}")

// 制限あり（スペースなしなら動作）
toki("{a+b}")
```

### エスケープシーケンス

| シーケンス | 文字 |
|-----------|------|
| `\n` | 改行 |
| `\t` | タブ |
| `\r` | キャリッジリターン |
| `\\` | バックスラッシュ |
| `\"` | ダブルクォート |
| `\{` | 左波括弧 |
| `\}` | 右波括弧 |

## 標準ライブラリ (ilo insa)

### I/O

| 関数 | 説明 |
|------|------|
| `toki(x, ...)` | 値を出力（改行付き） |

### 数値

| 関数 | 説明 |
|------|------|
| `nanpa_sin(s)` | 文字列を数値に変換 |
| `nanpa_len(n)` | 整数部の桁数 |

### 文字列

| 関数 | 説明 |
|------|------|
| `sitelen_len(s)` | 文字列の長さ（文字数） |
| `sitelen_sama(a, b)` | 文字列の比較（lon/ala） |

### リスト (kulupu)

| 関数 | 説明 |
|------|------|
| `kulupu_sin(...)` | リストを作成 |
| `kulupu_len(arr)` | リストの長さ |
| `kulupu_ken(arr, i)` | i番目の要素を取得（範囲外はala） |
| `kulupu_lon(arr, i, v)` | i番目にvを設定した新リストを返す |
| `kulupu_aksen(arr, v)` | vを追加した新リストを返す |

### マップ (nasin)

| 関数 | 説明 |
|------|------|
| `nasin_sin()` | 空のマップを作成 |
| `nasin_ken(m, key)` | keyの値を取得（なければala） |
| `nasin_lon(m, key, val)` | key:valを設定した新マップを返す |

## 予約語 (nimi awen)

以下の単語は識別子として使用できません:

```
la, open, pini, ilo, pana, wile, taso,
suli, lili, suli_sama, lili_sama, sama, jo, lon, ala
```

## コメント

```
// これはコメントです
x jo 42  // 行末コメント
```

## 真偽値の評価

以下は「偽」として扱われます:
- `ala`
- `0`
- `""` (空文字列)
- `[]` (空リスト)
- `{}` (空マップ)

それ以外は「真」として扱われます。

## ファイル拡張子

`.lipo`

## 実行方法

```bash
# ファイルを実行
lipona script.lipo

# コードを直接実行
lipona -e 'toki("pona")'
```
