# Phase 4: 力の多項式

## 目的

物体間の力（加速度への寄与）を、距離 d の符号付き整数乗の多項式で定義できるようにする。  
ニュートン重力（`+d^-3` 相当）をデフォルトとし、実験的な力則で銀河形成がどう変わるか観察する。

## 力の定義

ペア (i, j) に対し、i への加速度寄与（j の質量込み）:

```
a_ij = r̂ · Σ_k  sign_k · c_k · d^(N_k) · m_j
     = r   · Σ_k  sign_k · c_k · d^(N_k - 2) · m_j
```

- `r`: i → j の変位ベクトル（長さ d）
- `d`: ソフトニング込み距離 `sqrt(|r|² + softening²)`
- `sign_k ∈ {+1, -1}`, `N_k ∈ ℤ`, `c_k > 0`（係数）
- 項数上限: **8**

**デフォルト（ニュートン）**: 1 項 — `sign=+`, `N=-3`, `c=G`（現行 `gravity.wgsl` と同等）

**例**:
| 式（記法） | 意味 |
|-----------|------|
| `+d^-3` | 標準重力 |
| `+d^-3 -d^-2` | 重力 + 距離に反比例する斥力成分 |
| `-d^-1` | 斥力優位（発散しやすい — デモ用） |

UI 表記はユーザー要望の `[符号]d^[N]` 連結（例: UI 上 `+d^-3`）。

## 計画概要

### Task 1: 力則リソース
- `model/force.rs` — `ForceTerm`, `ForceLaw`, `pair_acceleration`
- Applied 値は `SimulationSettings.force`
- `ExtractResource` → `simulation/gpu/params.rs`

### Task 2: gravity.wgsl 一般化
- `Params` に term 配列 uniform を追加（`term_count`, `terms[]`）
- 内側ループ: 既存 j ループの中で k 項を累積
- `d^(N-2)` は `pow` または整数指数専用の分岐（N の範囲を UI で -5〜+2 等にクランプ）
- `N ≤ -2` で d→0 発散する項はソフトニング必須（警告表示）

### Task 3: 再起動時の加速度整合
- `model/force.rs::pair_acceleration` を `generate_initial_state` と Rust テストで共有
- 力則変更は `ui/apply.rs` 経由で `SimulationCommand::Restart`

### Task 4: 力則 UI
- `ui/panels/force.rs` — 折りたたみセクション「力の式」
- 項の追加・削除（最大 8）
- 各行: 符号トグル (+/−)、指数 N（整数スピン）、係数 c（スライダー）
- プレビュー文字列: `+G·d^-3` 形式
- プリセットドロップダウン（Newtonian 他 1〜2 個）
- 「Apply & Restart」

## 計画詳細

- **質量依存**: 全項で `m_j` を乗算（現行と同じ）。i 自身の質量は加速度計算側（integrate）で扱わない現行設計を維持。
- **性能**: 項数 8 × O(N²) は許容。将来タイル化は wasm_web Phase 2 の最適化に委ねる。
- **安定性**: |N| が大きい項や正の N（斥力）では爆発しやすい — UI に警告。係数・指数の hard clamp。
- **WGSL pow**: 整数指数は `switch` または展開で `pow` 回避可（Phase 4 着手時に決定）。
- **テスト**: Rust 側で 2 体・1 項 `-3` の加速度が `G·m/r²` 方向と一致することを単体テスト。

## 受け入れ条件

- [ ] デフォルト 1 項 `+d^-3` で Phase 3 完了時と同等の挙動
- [ ] UI で項を追加・符号反転・指数変更し Apply & Restart 後に挙動が変わる
- [ ] 2 項以上の多項式（例 `+d^-3 -d^-2`）が GPU 上で安定に動作する
- [ ] 力則プレビュー文字列が UI に表示される
- [ ] 不正な空の力則（term_count=0）は Apply 不可
- [ ] Rust 単体テスト: ニュートン 1 項と CPU 参照実装が一致
