# Slint × Rust アーキテクチャ要約

本ドキュメントは、本チャットで議論した **Slint + Rust における状態管理・イベント処理・MVVM/TEA的設計指針** を要約したものです。

---

## 1. 基本方針（結論）

* **Slint 側**: 宣言的UI・純粋なバインディング・派生状態の計算
* **Rust 側**: 状態管理・イベント処理・副作用・非純粋な処理
* **境界**: Rust → Slint は「関数」ではなく「値（property）」を渡す

👉 *「関数を呼ぶな、値を渡せ」*

---

## 2. イベントとデータフロー

推奨される一方向データフロー:

```
ユーザーイベント
  → Slint callback
    → Msg 発行
      → AppState::update(Msg)
        → projection
          → Slint property 更新
            → UI 再描画
```

* View から ViewModel / Rust への通知は **callback / Msg**
* UI 更新は **状態変化の結果** として行う

---

## 3. Slint と純粋性

* Slint の property バインディングは **pure function 前提**
* Rust 側の関数は Slint から見て純粋性を保証できない

### 非推奨

```slint
my-prop: my_callback_in_rust(other-prop);
```

### 推奨

```slint
my-prop: prop-from-rust;
```

```rust
// Rust 側で状態更新
prop_from_rust.set(new_value);
```

---

## 4. 共有状態の扱い

### 問題点（Observer パターン）

* 状態更新が分散
* 更新順序が不透明
* 因果関係が追いづらい

### 解決策

* **Single Source of Truth**
* コンポーネント間で共有される状態は 1 か所（AppState）に集約
* 更新は 1 か所（update）で行う

👉 Redux / TEA / Vue Store と同型の考え方

---

## 5. AppState / Msg / update の肥大化対策

### 正常な現象

* 中央集権型では必ず肥大化する
* 問題は「巨大さ」ではなく「分割軸」

### 対策

#### ① State を階層化

```rust
struct AppState {
    doc: DocState,
    ui: UiState,
}
```

#### ② Msg をスコープ分割

```rust
enum Msg {
    Doc(DocMsg),
    Ui(UiMsg),
}
```

#### ③ update を委譲

```rust
fn update(state: &mut AppState, msg: Msg) {
    match msg {
        Msg::Doc(m) => update_doc(&mut state.doc, m),
        Msg::Ui(m) => update_ui(&mut state.ui, m),
    }
}
```

---

## 6. ドメインオブジェクト（Doc）の扱い

### 非推奨

* Doc → Observer → 各モデルに直接通知

### 推奨

* Doc を **AppState に含める**
* UI へは **projection（投影）** を渡す

```rust
struct AppState {
    doc: Doc,
}
```

```rust
struct DocProjection {
    title: String,
    page_count: usize,
}
```

---

## 7. Slint Model（VecModel 等）を AppState に持つか？

### 原則（推奨）

* **AppState はフレームワーク非依存**
* Slint の `Model` は境界で生成

### 非推奨案

```rust
struct AppState {
    items: VecModel<Item>, // Slint 依存
}
```

### 妥協案（UI 専用 State）

```rust
struct AppState {
    doc: Doc,
}

struct UiState {
    items: Rc<VecModel<Item>>,
}
```

---

## 8. projection はどこで行うか

### 結論

* **update の直後、毎回行う**
* setup 時に 1 回だけ、は不十分

```rust
fn handle_msg(app: &mut AppState, ui: &Ui, msg: Msg) {
    update(app, msg);
    apply_projection(app, ui);
}
```

* TEA の `view(model)` 相当が `apply_projection`

---

## 9. TEA / Redux / Slint の対応関係

| 概念     | TEA         | Slint + Rust                |
| ------ | ----------- | --------------------------- |
| Model  | AppState    | AppState                    |
| Msg    | Msg         | Msg                         |
| update | update      | update                      |
| view   | view(model) | apply_projection(model, ui) |

---

## 10. 設計指針まとめ

1. **AppState は純 Rust・非 UI 依存**
2. **状態更新は Msg → update に集約**
3. **projection は毎回明示的に実行**
4. **Slint は純粋な View + 派生状態に専念**
5. **Observer はドメイン内部に閉じ込める**

---

## 11. 一言まとめ

> **Slint は「式の世界」、Rust は「現実世界」**
> **両者の境界を値でつなぐことで、安全で追いやすい UI アーキテクチャになる**
