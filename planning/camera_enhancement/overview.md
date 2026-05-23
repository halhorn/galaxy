# Story 3: カメラ拡張

## 目的

視点を自由に操作して、様々な角度・距離からシミュレーションを観察できるようにする。Story 1 の基本オービットカメラを拡張する。

## 現状

| 項目 | 状態 |
|------|------|
| カメラ | `bevy_panorbit_camera` 0.34（オービット・パン・ズーム） |
| 焦点 | 原点 `(0, 0, 0)` 固定 |
| ズーム | `target_radius` のみ変更（焦点方向への dolly）。カーソル位置は無視 |
| ビューポート | egui パネル除外の `SimulationViewportRect` + `SimulationCamera` |
| egui フォーカス | `EguiWantsFocus` / `EguiFocusIncludesHover` で入力ガード済み |

星団が物理演算で漂うと、ズーム軸（焦点）と観察対象がずれ、スクロール・ピンチ時に星が画面内を滑る。

## 計画概要

### Phase 1: [Zoom to cursor](phase1_zoom_to_cursor.md)

スクロール・ピンチの起点（カーソル／2 本指中点）の視線方向へ dolly し、画面上のその点を固定したまま拡大・縮小する。

### Phase 2: 追従・リセット（未着手）

物体追従、Frame All、Home キーリセットなど。Phase 1 完了後に別計画で詳細化。

## 依存関係

```
Story 1（基本 3D 表示・PanOrbitCamera）
  → Phase 1（Zoom to cursor）
  → Phase 2（追従・リセット）
```

Story 2 / Story 7 と独立して着手可能。

## 設計方針（確定）

| 項目 | 決定 |
|------|------|
| カメラモデル | オービットカメラを維持（Fly カメラは採用しない） |
| ライブラリ | `bevy_panorbit_camera` を継続。組み込みズームは無効化し自前 dolly を追加 |
| ピボット | カーソルレイと「現在の `focus` を通り視線に垂直な平面」の交点 |
| スムージング | 既存の `PanOrbitCamera` の `target_focus` / `target_radius` 補間をそのまま利用 |
| 配置 | `view/camera/`（3D カメラ concern。`ui` / `simulation` には置かない） |
| URL 同期 | カメラ姿勢は載せない（Story 7 と同方針） |

モジュール分割・依存ルールは [architecture.md](architecture.md) を参照。

## 受け入れ条件（Feature 全体）

- [ ] Phase 1: スクロール・ピンチでカーソル／中点下の点が画面上で固定されたままズームできる
- [ ] オービット・パン等の既存操作が壊れていない
- [ ] egui パネル上のスクロールが 3D ズームを誘発しない
- [ ] Phase 2: 物体追従・視点リセット（別 Phase で定義）
