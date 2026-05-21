# モジュール設計（Phase 0〜4 見通し）

## 目的

UI・表示・純粋なシミュレーションロジックを分離し、Phase 1〜4 を同じ骨格の上に載せる。  
Bevy の Main world（ゲームロジック）と Render world（GPU compute）の境界は維持する。

## レイヤー概要

```
┌─────────────────────────────────────────────────────────┐
│  ui/          操作パネル・ショートカット（bevy_egui）      │
│               Draft 編集 → Command 発行                    │
└───────────────────────────┬─────────────────────────────┘
                            │ read Resources / send Events
┌───────────────────────────▼─────────────────────────────┐
│  simulation/  Bevy ランタイム（再生制御・再起動・Extract）   │
└───────────────────────────┬─────────────────────────────┘
                            │ calls
┌───────────────────────────▼─────────────────────────────┐
│  model/       純粋ロジック（Bevy / GPU / UI 非依存）        │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  view/        3D 描画・選択マーカー（UI ではない）          │
│               GPU バッファ参照 + readback                   │
└───────────────────────────┬─────────────────────────────┘
                            │ read buffers / snapshot
                            └────────── simulation/ ──────┘

┌─────────────────────────────────────────────────────────┐
│  simulation/gpu/   Render world 専用 compute              │
└─────────────────────────────────────────────────────────┘
```

**依存ルール**

| From | To | 禁止 |
|------|-----|------|
| `ui` | `simulation`（Resource / Event のみ） | `gpu`, `view` 内部 |
| `view` | `simulation`, `model::constants` | `ui` |
| `simulation` | `model` | `ui`, `view` |
| `simulation/gpu` | `model`（uniform 変換のみ） | `ui`, `view` |
| `model` | （なし） | Bevy, wgpu |

---

## ディレクトリ・ファイル一覧

```
src/
├── main.rs
├── lib.rs                      # pub mod bootstrap, model, simulation, ui, view
│
├── bootstrap/
│   └── mod.rs                  # App 組み立て、Plugin 登録順
│
├── model/                      # ★ 純粋シミュレーションロジック
│   ├── mod.rs
│   ├── constants.rs            # コンパイル時上限・デフォルト
│   ├── body.rs                 # BodyArrays, active/inactive 判定
│   ├── physics.rs              # PhysicsSettings
│   ├── force.rs                # ForceTerm, ForceLaw, CPU 加速度
│   ├── initial.rs              # InitialConditions, 初期状態生成
│   └── rng.rs                  # SimpleRng
│
├── simulation/                 # ★ Bevy ランタイム + GPU ブリッジ
│   ├── mod.rs                  # SimulationPlugin
│   ├── playback.rs             # PlaybackState, 経過時間
│   ├── config.rs               # SimulationConfig（時間刻み）
│   ├── settings.rs             # Applied 設定の集約 Resource
│   ├── commands.rs             # SimulationCommand Event
│   ├── restart.rs              # 再起動システム
│   ├── upload.rs               # BodyArrays → GpuBuffers 書き込み
│   ├── shaders.rs              # WGSL 登録
│   └── gpu/                    # Render world 専用
│       ├── mod.rs              # SimulationGpuPlugin
│       ├── buffers.rs          # SimulationGpuBuffers
│       ├── params.rs           # GPU uniform 型 + model からの変換
│       ├── pipelines.rs        # パイプライン初期化
│       ├── bind_groups.rs      # 毎フレーム bind group 準備
│       └── node.rs             # SimulationComputeNode
│
├── view/                       # ★ 3D 表示（UI 以外）
│   ├── mod.rs                  # ViewPlugin
│   ├── bodies/
│   │   ├── mod.rs
│   │   ├── material.rs         # BodiesMaterial
│   │   ├── mesh.rs             # BodiesMesh, instanced mesh 構築
│   │   └── setup.rs            # 描画エンティティ spawn
│   └── selection/
│       ├── mod.rs
│       ├── snapshot.rs         # SimulationCpuSnapshot, readback
│       ├── pick.rs             # クリック選択
│       └── marker.rs           # 照準 gizmo
│
└── ui/                         # ★ 操作 UI
    ├── mod.rs                  # ControlUiPlugin
    ├── draft.rs                # ControlPanelDraft（未 Apply の編集中値）
    ├── keyboard.rs             # Space 等ショートカット
    ├── apply.rs                # Draft → settings + Command
    └── panels/
        ├── mod.rs              # egui フレーム・折りたたみ
        ├── playback.rs         # Phase 1
        ├── physics.rs          # Phase 2
        ├── initial.rs          # Phase 3
        └── force.rs            # Phase 4
```

`assets/shaders/` は現状どおり（`gravity.wgsl` 等）。Phase 4 で `gravity.wgsl` を多項式対応に変更。

---

## 主な型と責務

### `model/` — 純粋ロジック

| 型 | ファイル | 責務 |
|----|---------|------|
| `BODY_COUNT`, `MIN_MASS`, 各種 default / clamp 範囲 | `constants.rs` | バッファ上限・不変の物理閾値 |
| `BodyArrays` | `body.rs` | `positions`, `velocities`, `masses`, `accelerations` の Vec。`active_count` |
| `is_active(mass)` | `body.rs` | `mass > MIN_MASS` |
| `PhysicsSettings` | `physics.rs` | `g`, `softening`, `merge_radius_factor`, `merge_inv_cell_size()` |
| `ForceTerm` | `force.rs` | `{ sign: i8, exponent: i32, coefficient: f32 }` |
| `ForceLaw` | `force.rs` | 最大 8 項、`term_count`, `newtonian_default()` |
| `pair_acceleration(...)` | `force.rs` | 1 ペアの加速度寄与（CPU 参照実装・再起動時の初期 acc 用） |
| `ForceLaw::display_string()` | `force.rs` | UI プレビュー `+G·d^-3` |
| `InitialConditions` | `initial.rs` | seed, n_stars, 円盤幾何, `active_count` |
| `generate_initial_state(...)` | `initial.rs` | `PhysicsSettings` + `ForceLaw` + `InitialConditions` → `BodyArrays` |
| `SimpleRng` | `rng.rs` | wasm 安全な決定論 RNG |

**テスト**: `model/force.rs`, `model/initial.rs` に `#[cfg(test)]` ユニットテスト（Bevy 不要）。

---

### `simulation/` — ランタイム

| 型 | ファイル | 責務 |
|----|---------|------|
| `SimulationPlugin` | `mod.rs` | サブ Plugin 登録、Startup で初回 `Restart` |
| `PlaybackState` | `playback.rs` | `Running \| Paused`, `accumulated_sim_time` |
| `SimulationConfig` | `config.rs` | `time_scale`, `fixed_dt`, `dt()` — ExtractResource |
| `SimulationSettings` | `settings.rs` | Applied 値の単一 Resource: physics + initial + force + active_count |
| `SimulationCommand` | `commands.rs` | `Restart`, `ApplyPhysics`, `ApplyInitial`, `ApplyForce` |
| `restart_simulation` | `restart.rs` | Command 処理: pause → `generate_initial_state` → upload → snapshot 無効化 |
| `tick_sim_time` | `playback.rs` | Running 時のみ `accumulated_sim_time += dt()` |
| `SimulationGpuBuffers` | `gpu/buffers.rs` | GPU SSBO Handles — ExtractResource |
| `GravityUniform`, `IntegrateUniform`, `MergeUniform` | `gpu/params.rs` | WGSL 対応 struct + `From<&SimulationSettings>` |
| `SimulationComputeNode` | `gpu/node.rs` | compute pass。`PlaybackState::Running` のときのみ dispatch |
| `SimulationGpuPlugin` | `gpu/mod.rs` | RenderApp 登録、ExtractResource、render graph |

**Applied vs Draft**: Main world には **Applied** の `SimulationSettings` のみ保持。UI の Draft は `ui/draft.rs` に閉じる。

---

### `view/` — 3D 表示

| 型 | ファイル | 責務 |
|----|---------|------|
| `ViewPlugin` | `mod.rs` | bodies + selection Plugin |
| `BodiesMaterial`, `BodiesMesh` | `bodies/` | instanced mesh 描画。position/mass バッファ参照 |
| `setup_bodies_render` | `bodies/setup.rs` | 描画エンティティ 1 体 spawn |
| `SimulationCpuSnapshot` | `selection/snapshot.rs` | readback ミラー。`ready` フラグ |
| `SelectedBody` | `selection/pick.rs` | 選択 index |
| `click_pick_body`, `draw_selection_marker` | `selection/` | 入力 → 選択、gizmo 描画 |

再起動後: `restart` が snapshot の `ready = false` にし、readback 完了で再び pick 可能に。

---

### `ui/` — 操作 UI

| 型 | ファイル | 責務 |
|----|---------|------|
| `ControlUiPlugin` | `mod.rs` | bevy_egui + keyboard + apply systems |
| `ControlPanelDraft` | `draft.rs` | 全パネルの編集中値。起動時は `SimulationSettings` から clone |
| `draw_control_panel` | `panels/mod.rs` | egui ウィンドウ、セクション折りたたみ |
| `playback_panel` | `panels/playback.rs` | 停止/再開、時間倍率、経過時間表示 |
| `physics_panel` | `panels/physics.rs` | G, softening, merge — Phase 2 |
| `initial_panel` | `panels/initial.rs` | seed, n_stars, active_count — Phase 3 |
| `force_panel` | `panels/force.rs` | 多項式項編集 — Phase 4 |
| `apply_draft` | `apply.rs` | Apply ボタン → validate → `SimulationCommand` |
| `playback_shortcuts` | `keyboard.rs` | Space トグル等 |

UI は `SimulationSettings`, `PlaybackState`, `SimulationConfig` を **読む**だけ。GPU / `BodyArrays` には触れない。

---

## データフロー

### 通常フレーム（Running）

```
PlaybackState=Running
  → Extract: SimulationSettings, SimulationConfig, SimulationGpuBuffers, PlaybackState
  → gpu/node: position → gravity → velocity → merge
  → view/bodies: バッファ参照のまま描画
  → view/selection: readback 更新 → pick / marker
  → ui: 状態表示のみ
```

### Apply & Restart（Phase 3/4）

```
ui Apply
  → SimulationCommand::Restart { settings }
  → simulation/restart:
       PlaybackState = Paused
       BodyArrays = model::generate_initial_state(...)
       upload → SimulationGpuBuffers
       SimulationCpuSnapshot.ready = false
  → ユーザーが Resume
```

### Apply 物理のみ（Phase 2）

```
ui ApplyPhysics
  → SimulationSettings.physics 更新
  → Extract → gpu/params が次フレーム uniform 更新
  （再起動不要、加速度は次 step から新力則）
```

---

## Phase 対応表

| Phase | 主に触るモジュール |
|-------|-------------------|
| 0 | 全体リファクタ（下記） |
| 1 | `simulation/playback`, `ui/panels/playback`, `ui/keyboard`, `gpu/node` ゲート |
| 2 | `model/physics`, `simulation/settings`, `ui/panels/physics`, `gpu/params` |
| 3 | `model/initial`, `model/body`, `simulation/restart`, `ui/panels/initial` |
| 4 | `model/force`, `gpu/params`, `assets/shaders/gravity.wgsl`, `ui/panels/force` |

---

## Plugin 登録順（`bootstrap/mod.rs`）

```text
DefaultPlugins
PanOrbitCameraPlugin
SimulationPlugin      # playback, settings, restart, gpu, shaders
ViewPlugin              # bodies, selection
ControlUiPlugin         # egui — View より後（入力優先の調整）
```

---

## 現行ファイルからの移行対応

| 現行 | 移行先 |
|------|--------|
| `simulation/constants.rs` | `model/constants.rs` |
| `simulation/init.rs` | `model/initial.rs` + `simulation/restart.rs` |
| `simulation/config.rs` | `simulation/config.rs`（そのまま） |
| `simulation/buffers.rs` | `simulation/gpu/buffers.rs` |
| `simulation/compute.rs` | `simulation/gpu/{params,pipelines,bind_groups,node}.rs` |
| `simulation/shaders.rs` | `simulation/shaders.rs` |
| `simulation/render.rs` | `view/bodies/*` |
| `simulation/selection.rs` | `view/selection/*` |
| `simulation/mod.rs` | `simulation/mod.rs`（薄い集約） |

---

## 受け入れ条件（設計）

- [ ] `model/` に `bevy` / `wgpu` / `egui` の import がない
- [ ] `ui/` から `simulation/gpu/` を import していない
- [ ] `view/` から `ui/` を import していない
- [ ] Applied 設定は `SimulationSettings` 1 箇所、Draft は `ControlPanelDraft` のみ
- [ ] 再起動ロジックは `simulation/restart.rs` のみ（UI は Command 発行のみ）
- [ ] CPU 力計算と GPU 力計算のパラメータ源は同じ `ForceLaw` / `PhysicsSettings`
