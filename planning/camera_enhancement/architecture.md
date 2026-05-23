# モジュール設計（Zoom to cursor）

## 目的

`bevy_panorbit_camera` の半径のみズームを、カーソル起点 dolly に置き換える。既存の `view` / `simulation` / `ui` レイアウト（[simulation_control/architecture.md](../simulation_control/architecture.md)）を壊さず、カメラ入力 concern を `view/camera/` に集約する。

---

## レイヤー概要

```
┌─────────────────────────────────────────────────────────┐
│  ui/           egui パネル。カメラ入力は触らない           │
└───────────────────────────┬─────────────────────────────┘
                            │ EguiWantsFocus（既存）
┌───────────────────────────▼─────────────────────────────┐
│  view/camera/  ズーム入力 → pivot 計算 → PanOrbitCamera   │
│                の target_focus / target_radius 更新       │
└───────────────────────────┬─────────────────────────────┘
                            │ 更新後
┌───────────────────────────▼─────────────────────────────┐
│  bevy_panorbit_camera   pan_orbit_camera（補間・Transform）│
└───────────────────────────┬─────────────────────────────┘
                            │ renders
┌───────────────────────────▼─────────────────────────────┐
│  view/（bodies, selection）  SimulationCamera + viewport  │
└─────────────────────────────────────────────────────────┘
```

**依存ルール**

| From | To | 禁止 |
|------|-----|------|
| `view/camera/` | `view::SimulationCamera`, `simulation::SimulationViewportRect`, `bevy_panorbit_camera` | `ui`, `simulation/gpu`, `model` |
| `view/` | `view/camera/`（Plugin 登録のみ） | — |
| `bootstrap/` | `view/camera` の Plugin、`PanOrbitCamera` 初期設定 | camera 内部ロジック直接 import |
| `ui/` | （変更なし） | `view/camera` |

---

## ディレクトリ・ファイル一覧

```
src/
├── view/
│   ├── mod.rs                      # CameraPlugin 登録を追加
│   ├── sim_viewport.rs             # 変更なし（SimulationCamera, viewport 更新）
│   └── camera/
│       ├── mod.rs                  # CameraControlsPlugin
│       ├── pivot.rs                # ピボット計算（純関数、単体テスト可）
│       └── zoom_to_cursor.rs       # 入力収集 + target 更新システム
│
└── bootstrap/mod.rs                # PanOrbitCamera { zoom_sensitivity: 0.0, … }
```

---

## 公開 API（責務レベル）

| 名前 | 種別 | 責務 |
|------|------|------|
| `CameraControlsPlugin` | Plugin | ズーム to cursor システムを `PostUpdate` に登録 |
| `zoom_pivot_on_focus_plane` | fn | カメラレイと focus 深度平面の交点（ピボット）を返す |
| `apply_zoom_to_pivot` | fn | スクロール量・ピボットから `target_focus` / `target_radius` の増分を計算 |
| `zoom_to_cursor_system` | system | 入力・ガード・上記 fn 適用 |

---

## システム順序

```
PostUpdate:
  EguiPreUpdateSet::InitContexts
    → check_egui_wants_focus（bevy_panorbit_camera）
    → zoom_to_cursor_system          ← .before(PanOrbitCameraSystemSet)
    → PanOrbitCameraSystemSet        ← 補間・Transform 反映
    → TransformSystems::Propagate
```

自前ズームは **必ず `PanOrbitCameraSystemSet` より前** に `target_*` を書き込む。組み込みズームは `zoom_sensitivity: 0.0` で無効化し二重適用を防ぐ。

---

## 入力ソース

| 入力 | スクリーン座標 | 備考 |
|------|----------------|------|
| `MouseWheel`（line / pixel） | `Window::cursor_position()` | 既存 panorbit と同量感の delta 換算 |
| `PinchGesture` | ジェスチャ中心（要変換） | macOS トラックパッド等 |
| 2 本指ピンチ（`Touches`） | 2 点の中点 | `zoom_sensitivity: 0.0` で panorbit 側ピンチも停止するため自前で処理 |

---

## 座標系

- カーソル → レイ: `Camera::viewport_to_world`（[pick.rs](../../src/view/selection/pick.rs) と同パターン）
- ビューポート外・egui 上は処理しない
- `SimulationViewportRect` は参照のみ（カメラ `viewport` は `sim_viewport` が既に設定済み）

---

## テスト方針

| 対象 | 方法 |
|------|------|
| `pivot.rs` | 純関数の `#[cfg(test)]`（レイ・平面交差、カメラ後方など） |
| `apply_zoom_to_pivot` | 純関数テスト（ズーム前後で pivot のスクリーン投影が不変、など） |
| 統合 | 手動（デスクトップ + モバイル/WASM）。自動 E2E は今回スコープ外 |
