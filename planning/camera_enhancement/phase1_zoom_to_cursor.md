# Phase 1: Zoom to cursor

## 目的

ホイール・トラックパッドスクロール・2 本指ピンチで、操作起点の視線方向へ dolly する。星が漂った状態でも、ユーザーが指した位置を軸に自然に拡大・縮小できるようにする。

## 前提

- [architecture.md](architecture.md) のモジュール配置・システム順序
- `bevy_panorbit_camera` 0.34 の `PanOrbitCameraSystemSet` / `EguiWantsFocus`
- 既存の `SimulationCamera` + 部分 viewport

## 計画概要

### Task 1: 基盤 — 組み込みズーム無効化と Plugin 骨格

`view/camera/` 追加、`bootstrap` で `zoom_sensitivity: 0.0`、`CameraControlsPlugin` 登録。

### Task 2: ピボット計算（`pivot.rs`）

カーソルレイと focus 深度平面の交点。エッジケース（平行・カメラ後方）のフォールバック。

### Task 3: Dolly 適用（`zoom_to_cursor.rs`）

スクロール／ピンチ delta を panorbit 互換の radius 変化に換算し、`target_focus` と `target_radius` を pivot 方向へ同時更新。

### Task 4: 入力統合とガード

`MouseWheel`・`PinchGesture`・2 本指 `Touches`。egui フォーカス・ビューポート外をスキップ。

### Task 5: 手動検証

漂った星団・ビューポート端・モバイルピンチの確認。

## 計画詳細

### ズーム算法（確定）

1. スクリーン座標 `s` からカメラレイ `(origin, dir)` を取得
2. 平面: 法線 = カメラ forward、`focus` を通過
3. 交点 `pivot = zoom_pivot_on_focus_plane(...)`
4. radius 変化量 `Δr` を panorbit 既存式に合わせる（`line` / `pixel` / pinch）
5. 比率 `α = Δr / current_radius`（符号付き）で  
   `target_focus ← focus + (pivot - focus) * α`  
   `target_radius ← radius + Δr`（limit 適用は panorbit に委譲）

焦点のみ動かさず radius だけ変える現行挙動との差分は、**pivot 方向への focus シフト**。

### panorbit 無効化

- `PanOrbitCamera::default()` に `zoom_sensitivity: 0.0` を設定
- スクロール・タッチピンチ・`PinchGesture` すべて自前システムで処理（二重ズーム防止）
- オービット・パン・`zoom_lower_limit` 等は変更しない

### 感度

- 初期値: panorbit 0.34 の `line_delta` / `pixel_delta` 式（`target_radius * 0.2` スケール）を流用
- pinch: panorbit touch 換算（`pinch * 0.015` 系）を参考に同オーダーで調整

### スコープ外（Phase 1）

- 重心（COM）自動追従
- 物体追従・Frame All
- カメラ姿勢の URL 同期
- `bevy_panorbit_camera` のフォークや `bevy_blendy_cameras` への乗り換え

## 受け入れ条件

### Task 1

- [ ] `view/camera/mod.rs` と `CameraControlsPlugin` が存在し `ViewPlugin` から登録される
- [ ] `bootstrap` の `PanOrbitCamera` で `zoom_sensitivity: 0.0`
- [ ] ビルドが通る

### Task 2

- [ ] `zoom_pivot_on_focus_plane` の単体テストが通る
- [ ] レイが平面と平行な場合に `focus` フォールバック（クラッシュしない）

### Task 3

- [ ] `apply_zoom_to_pivot` の単体テストが通る
- [ ] ズーム適用後も yaw / pitch は変えず、focus + radius のみ更新

### Task 4

- [ ] シミュレーション viewport 内のホイールで zoom to cursor が動作
- [ ] egui パネル上のスクロールでは 3D ズームしない
- [ ] 2 本指ピンチ（タッチ）で中点を pivot にズーム
- [ ] トラックパッド `PinchGesture` があれば同様に動作

### Task 5（手動）

- [ ] 星団が原点から漂った状態で、星の上を指してスクロール → その星が画面上でほぼ固定されたまま拡大・縮小
- [ ] ビューポート端付近でも破綻しない（フォールバックまたは許容範囲内のずれ）
- [ ] 左ドラッグオービット・右ドラッグパンが従来どおり
- [ ] デスクトップ（ホイール）とモバイル/WASM（ピンチ）の少なくとも一方で確認
