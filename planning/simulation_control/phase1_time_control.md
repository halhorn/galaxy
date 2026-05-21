# Phase 1: 時間制御 + UI 基盤

## 目的

シミュレーションの再生状態をユーザーが制御できるようにする。以降の Phase で増えるパラメータ UI の土台を作る。

## 計画概要

### Task 1: 再生状態リソース
- `simulation/playback.rs` — `PlaybackState` リソース: `Running | Paused`
- `SimulationConfig.time_scale` を早送り倍率として利用（既存 `simulation/config.rs`）
- `simulation/gpu/node.rs` の `SimulationComputeNode` が `PlaybackState::Running` のときのみ dispatch

### Task 2: 入力・ショートカット
- `ui/keyboard.rs`: Space で Running ↔ Paused
- 数字キーまたは UI: 時間倍率プリセット（0.25x, 1x, 2x, 4x）

### Task 3: bevy_egui 導入
- `bevy_egui` を依存に追加
- `ui/mod.rs` — `ControlUiPlugin`
- `ui/panels/mod.rs` — 画面左上の折りたたみパネル
- 表示: 再生状態、`PlaybackState::accumulated_sim_time`、FPS、時間倍率

### Task 4: 基本コントロール UI
- `ui/panels/playback.rs`: 一時停止 / 再開、時間倍率スライダー
- Phase 2〜4 用セクションは `panels/physics.rs` 等を空 stub で配置

## 計画詳細

- **経過時間**: `accumulated_sim_time` を Main world で `dt()` 積分（Running 時のみ）。Render world には不要。
- **Pause 中の描画**: compute を止めても position バッファは最後の状態のまま描画継続。
- **カメラとの競合**: egui がポインタを capture したとき PanOrbitCamera が反応しないよう、egui の入力優先を確認。

## 受け入れ条件

- [ ] Space または UI ボタンで一時停止・再開が切り替わる
- [ ] 一時停止中は物体が静止し、再開後に同じ位置から続行する
- [ ] 時間倍率 0.25x〜4x を UI で変更でき、物理ステップ `dt` に反映される
- [ ] パネルに再生状態と時間倍率が表示される
- [ ] パネル操作中もカメラ操作が可能（パネル外クリック時）
