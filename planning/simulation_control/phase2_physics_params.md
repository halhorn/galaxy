# Phase 2: ランタイム物理パラメータ

## 目的

コンパイル時定数になっている物理パラメータを実行中に変更し、銀河形成への影響を観察できるようにする。

## 対象パラメータ

| パラメータ | 現状 | 役割 |
|-----------|------|------|
| `G` | `constants.rs` | 重力の強さ |
| `SOFTENING` | `constants.rs` | 近距離での力の発散防止 |
| `MERGE_RADIUS_FACTOR` | `constants.rs` → `merge.wgsl` | 衝突判定距離（半径比） |
| `time_scale` | `SimulationConfig` | Phase 1 で UI 化済み |

## 計画概要

### Task 1: 物理設定リソース
- `model/physics.rs` — `PhysicsSettings`（純粋データ + merge_inv_cell_size 等）
- `simulation/settings.rs` — Applied 値として `SimulationSettings.physics` に保持
- `ExtractResource` → `simulation/gpu/params.rs` が `GravityUniform` / `MergeUniform` を構築

### Task 2: UI コントロール
- `ui/panels/physics.rs` — 折りたたみセクション「物理」
- スライダー + 数値入力:
  - G: 対数スケール推奨（例 1〜100, デフォルト 4π²）
  - Softening: 0.001〜0.1 AU
  - Merge radius factor: 0.05〜0.5
- 「Apply」ボタン: 変更を確定（Running 中は Apply 時に一時停止 → uniform 更新 → 再開オプション）

### Task 3: ソフトニング変更時の整合
- `softening_sq` は CPU 側で計算して uniform に渡す
- 初期加速度は Phase 3 まで再計算しない（力変更は Phase 4）。G / softening 変更時は次フレームから新パラメータが効く

## 計画詳細

- **即時反映 vs Apply**: スライダー操作中はプレビュー表示のみ、Apply で GPU uniform 更新。誤操作防止とパフォーマンスのため。
- **MERGE_CELL_SIZE**: `merge_radius_factor` 変更に伴い `inv_cell_size` を CPU で再計算（`MERGE_MAX_RADIUS` は当面固定）。
- **範囲**: UI の min/max は定数モジュールにデフォルト範囲として定義。極端な値での NaN は Phase 4 以前は許容（クランプ優先）。

## 受け入れ条件

- [ ] UI から G・ソフトニング・マージ距離を変更し Apply 後に挙動が変わる
- [ ] マージ距離を大きくすると合体が早く、小さくすると離散星が増える
- [ ] ソフトニングを大きくすると中心付近の散乱が穏やかになる
- [ ] Apply 前のスライダー操作だけでは GPU パラメータが変わらない
- [ ] デフォルト値 Apply 後、Phase 1 以前と視覚的に同等の挙動に戻る
