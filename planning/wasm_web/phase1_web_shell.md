# Phase 1: Web シェル

## 目的

Web 専用のビルド基盤と Bevy WebGPU シェルを整え、ブラウザ上でキャンバス + カメラ操作が動く状態にする。

## 計画概要

### Task 1: プロジェクト構成の整理
Web 専用に Cargo を整理する。

- `webgpu = ["bevy/webgl2", "bevy/webgpu"]` ではなく **`webgpu = ["bevy/webgpu"]` のみ**
- `[profile.wasm-release]`（`opt-level = "s"`, `lto`, `codegen-units = 1`, `strip`）
- `[target.'cfg(target_arch = "wasm32")'.dependencies]`: `wasm-bindgen`, `web-sys`, `js-sys`（最小 feature）
- ネイティブ専用依存（`pollster`、独立 `wgpu` 等）は削除
- 既存 `src/` は Phase 2 で置き換える前提で、Phase 1 時点では最小 Bevy アプリに差し替え可

### Task 2: Trunk 設定
- ルート `Trunk.toml`（`target = "index.html"`, `dist = "dist"`）
- `.gitignore` に `dist/`
- ビルド: `RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk serve`
- Trunk HTML: `data-cargo-features="webgpu"`, `data-wasm-opt="z"`

### Task 3: index.html
fractalium を簡略化した版。

- フルスクリーン `<canvas id="galaxy-canvas">`
- ローディング `#galaxy-loading`
- WebGPU 非対応用 `#galaxy-no-webgpu`（JS または Rust から表示）
- タッチ・ズーム抑止 CSS

### Task 4: Bevy 起動
- `src/bootstrap/mod.rs` に App 組み立て
- `Window`: `canvas`, `fit_canvas_to_parent`, `prevent_default_event_handling`
- `PanOrbitCamera` + `Camera3d`
- Startup でローディング UI を非表示

## 計画詳細

**WebGPU 必須**: Chrome / Edge は実用可。Firefox / Safari は要検証。非対応時は CPU フォールバックせずエラー表示。

**開発フロー**: `trunk serve` のみ。`cargo run`（ネイティブ）は不要。

**参考（fractalium）**: `Trunk.toml`, `index.html` の構造。feature は `webgl2` ではなく `webgpu`。

## 受け入れ条件

- [ ] `RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk serve` が成功する
- [ ] ブラウザに canvas が表示され、マウスでカメラが動く
- [ ] ローディング UI が起動後に消える
