# Phase 0: モジュール分割リファクタ

## 目的

Phase 1 着手前に、現行の `src/simulation/` モノリスを [architecture.md](architecture.md) のレイヤーに分割する。  
**挙動変更なし**（リファクタのみ）。

## 計画概要

### Task 1: `model/` 抽出
- `constants.rs` を移動（`G` 等の **デフォルト値** は残し、ランタイム値は Phase 2 で `PhysicsSettings` へ）
- `init.rs` の生成ロジック → `initial.rs` + `rng.rs`
- `BodyArrays` 型を導入し、生成結果を Vec バンドルとして返す
- `generate_initial_state(ic, physics, force)` — Phase 4 まで force は Newton 固定でよい

### Task 2: `simulation/gpu/` 分割
- `buffers.rs` 移動
- `compute.rs` を `pipelines.rs`, `bind_groups.rs`, `node.rs`, `params.rs` に分割
- `GravityParams` 等は `params.rs` に集約

### Task 3: `view/` 抽出
- `render.rs` → `view/bodies/`
- `selection.rs` → `view/selection/`
- `ViewPlugin` 新設

### Task 4: `simulation/` ランタイム整理
- `spawn_initial_state` Startup → `restart.rs` + 初回 `Restart` 相当
- `upload.rs`: `BodyArrays` → `ShaderStorageBuffer` write
- `SimulationPlugin` が gpu + playback（空 stub）+ restart を登録
- `settings.rs`: 起動時デフォルトの `SimulationSettings`（現行定数から構築）

### Task 5: 配線更新
- `lib.rs` に `model`, `view` を追加
- `bootstrap/mod.rs` で `ViewPlugin` 登録
- `cargo check` / 手動で Web 起動確認

## 計画詳細

- **分割単位**: 1 PR = Phase 0 全体でも可。コンパイルが通る状態を各 Task 末で維持。
- **命名**: 現行の public 型名（`SimulationGpuBuffers`, `BodiesMesh` 等）は可能な限り維持し、呼び出し側 diff を最小化。
- **テスト**: Task 1 完了時に `model/force.rs` の Newton 1 項テストを追加（Phase 4 前の足場）。

## 受け入れ条件

- [ ] ディレクトリ構成が [architecture.md](architecture.md) と一致
- [ ] リファクタ前後でシミュレーション見た目・挙動が同等
- [ ] `model/` が Bevy 非依存
- [ ] `cargo clippy` / `cargo check` 通過
- [ ] Phase 1 で追加するファイルの **空 stub 置き場**（`ui/mod.rs`, `playback.rs` 等）が存在
