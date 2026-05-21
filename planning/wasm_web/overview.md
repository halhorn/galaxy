# Story 6: Web 専用 GPU シミュレータ

## 目的

Gravitium を **Web 専用** の 3D N 体重力シミュレータとして再設計し、ブラウザ上で **10,000 体** を GPU compute で動かす。  
ネイティブ対応・既存コードの互換は求めない。可能な限りシンプルな構成を優先する。

参考: [fractalium](https://github.com/halhorn/fractalium) の Trunk / GitHub Pages パターン（ただし WebGL2 ではなく **WebGPU** を使用）。

## 方針（確定）

| 項目 | 決定 |
|------|------|
| ターゲット | Web のみ（`wasm32-unknown-unknown`） |
| 公開 URL | `https://halhorn.github.io/gravitium/`（`--public-url /gravitium/`） |
| 物体数 | 10,000 |
| 物理 | GPU compute 必須（CPU フォールバックなし） |
| レンダリング | Bevy + WebGPU |
| ECS | カメラ・設定のみ。10,000 体は ECS エンティティにしない |
| 衝突マージ | Phase 2 Task 5（旧 `merger.rs` と同仕様） |

## アーキテクチャ概要

```
index.html (canvas)
    ↓
Bevy（薄いホスト）
  - キャンバス接続・入力・PanOrbitCamera
  - instanced mesh 描画（position バッファを参照）
    ↓
GPU 常駐シミュレーション
  - buffers: positions, velocities, masses, accelerations
  - compute: 加速度計算 → Velocity Verlet 積分
  - CPU readback なし（物理ループは GPU 完結）
```

**捨てるもの**: 独立 `wgpu::Instance`、`pollster`、CPU `NewtonianGravity`、`ForceCalculator` trait（初版）、10,000 `Mesh3d` エンティティ、ECS ベースの `merger.rs` 実装（仕様は Phase 2 で GPU 化）。

## 計画概要

### Phase 1: [Web シェル](phase1_web_shell.md)
Trunk + WebGPU + canvas。Bevy が起動し、カメラ操作できる空画面まで。

### Phase 2: [GPU シミュレーション本体](phase2_gpu_simulation.md)
GPU バッファ初期化、compute パイプライン、描画、衝突マージ。10,000 体が動く。

### Phase 3: [デプロイ](phase3_deploy.md)
GitHub Pages 公開。WebGPU 非対応時のエラー UI。

## 依存関係

```
Phase 1 → Phase 2 → Phase 3
```

Phase 2 がプロジェクトの核心。Phase 1 完了後に Phase 2 へ、Phase 2 完了後に Phase 3 へ。

## 受け入れ条件（Story 全体）

- [ ] `trunk serve` で WebGPU 上 10,000 体の重力シミュレーションが 3D 表示される
- [ ] マウスでオービットカメラが動く
- [ ] 30fps 以上（目標 60fps）
- [ ] WebGPU 非対応ブラウザで起動失敗メッセージが表示される
- [ ] `https://halhorn.github.io/gravitium/` からアクセスできる
