# 📘 Lipona Programming Language – MVP Specification (v0.1)

## 0. 概要

Lipona は **Toki Pona の文構造をベースにしたミニマルなプログラミング言語**である。  
構文を可能な限り少なく保ち、機能の拡張は **新しい構文ではなく関数（ilo）で行う**ことを哲学とする。

目的：
- 読むだけで意味が通るコード
- 学習コストの低さと表現力の両立
- 言語の肥大化（構文地獄）を回避
- Lisp 的抽象化の強さを Toki Pona の文法で実現

---

## 1. 基本文法

### 1.1 ソース文字
- ローマ字（ASCII）のみ
- Unicode・sitelen pona 記述は将来拡張対象

### 1.2 コメント

// 行末までコメントアウト

### 1.3 識別子（変数・関数名）

[a-zA-Z_][a-zA-Z0-9_]*

※命名は **トキポナ語でも英語でもどちらでもよい**。  
識別子がラテン文字規則を満たせば許可する。

例（全て合法）：

ilo sum
ilo sine_wave
ilo kulupu_nasin_sin
ilo json_parse

### 1.4 予約語（識別子に使用不可）

li, e, la, open, pini, ilo, pali, pana,
wile, taso,
suli, lili, sama,
jo, lon, ala

---

## 2. 型とリテラル

種類 / 表記
- 数値: 10, 3.14
- 文字列: "pona"
- 真偽: lon（true）, ala（false/null）

ala は false/null に相当するボトム値として扱う。

---

## 3. 式（Expression）

- 演算子：+ , - , * , /
- 優先順位：
  1. ()
  2. * /
  3. + -
     左結合

- 関数呼び出し（式として使用可能）

NAME e (arg1, arg2, ...)

例：

x li jo e sum e (a, b)
toki e sine_wave e (440, 2)

---

## 4. ステートメント（Statement）

### 4.1 代入

x li jo e Expr

### 4.2 比較

x li suli e y    // x > y
x li lili e y    // x < y
x li sama e y    // x == y

---

## 5. 制御構文

### 5.1 if / else

Condition la open
    Stmt*
pini
taso open
    Stmt*
pini

Condition に使えるのは：
- lon / ala
- 真偽値を格納する変数
- 比較式（例：x li suli e y）

### 5.2 while

wile Condition la open
    Stmt*
pini

---

## 6. 関数

### 6.1 関数定義

ilo NAME li pali e (param1, param2...) la open
    Stmt*
pini

### 6.2 return

pana e Expr

pana e が実行されなかった場合の戻り値は ala。

---

## 7. 標準ライブラリ（MVP最小セット）

※ 全て通常の関数（ilo）として提供。  
※シンタックスの追加は行わない。

### 7.1 入出力

- toki e (x) : print

### 7.2 数値

- nanpa_sin e (x) : 文字列 → 数値変換
- nanpa_len e (x) : 数字の桁数

### 7.3 文字列

- sitelen_len e (s) : 長さ
- sitelen_sama e (a, b) : 同値判定

### 7.4 リスト

- kulupu_sin e (...items) : リスト生成
- kulupu_len e (arr) : 長さ
- kulupu_ken e (arr, i) : 要素取得
- kulupu_lon e (arr, i, val) : 要素代入
- kulupu_aksen e (arr, val) : append

### 7.5 マップ

- nasin_sin e () : 空マップ生成
- nasin_ken e (m, key) : get
- nasin_lon e (m, key, val) : set

---

## 8. エラー仕様

- 未定義変数参照: 即時エラー pakala
- 0除算: pakala
- 型矛盾（例：文字列 * 数値）: pakala
- 存在しないキーの取得: ala を返す
- 存在しないキーへの代入: pakala

---

## 9. サンプルコード

### 9.1 Hello World

toki e ("pona mute!")

### 9.2 関数

ilo sum li pali e (a, b) la open
    pana e a + b
pini

x li jo e sum e (10, 20)
toki e (x)

### 9.3 while

i li jo e 0
wile i li lili e 5 la open
    toki e (i)
    i li jo e i + 1
pini

### 9.4 リスト操作

nums li jo e kulupu_sin e (1,2,3)
toki e (kulupu_len e (nums))
kulupu_lon e (nums, 1, 99)
toki e (kulupu_ken e (nums, 1))

---

## 10. AST 概要（実装者向け）

(例: Rust)

pub enum Expr {
    Number(f64),
    String(String),
    Bool(bool),
    Var(String),
    Binary { left: Box<Expr>, op: BinOp, right: Box<Expr> },
    FuncCall { name: String, args: Vec<Expr> },
}

pub enum Stmt {
    Assign { target: Expr, expr: Expr },
    If { cond: Expr, then_block: Block, else_block: Option<Block> },
    While { cond: Expr, block: Block },
    FuncDef { name: String, params: Vec<String>, body: Block },
    Return(Expr),
    Expr(Expr),
}

pub type Block = Vec<Stmt>;

---

## 11. 拡張候補（MVP外で追加可能）

- 例外処理: pakala の受け取り
- モジュールインポート: kepeken など
- パターンマッチ: 追加しない方向
- JIT / LLVM: 実装次第
- sitelen pona 記述: 将来目標
- 型システム: 実用拡張時に検討

---

## 12. まとめ

Lipona は **Toki Pona の文法構造を核に置き、構文拡張ではなく関数による抽象化で強くなるミニマル言語**である。

- コア構文は小さく、直感的で、統一的
- 標準ライブラリがプログラミング的表現力を補う
- 名前はトキポナ語でも英語でも自由
- 学習者・研究者・実装者・哲学的探究者すべてが使える

目的：読むだけで意味が通るプログラミング
手段：構文を増やさず、関数で拡張する

