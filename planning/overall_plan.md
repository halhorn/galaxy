# Gravitium — 重力シミュレーションプロジェクト

## 目的

3D空間で物体同士が任意の力の式で相互作用する N体シミュレータを構築する。
1000体規模でリアルタイム (60fps) 動作し、3D ビジュアル・インタラクティブに結果を観察できるようにする。

## 技術スタック

| 項目 | 選定 |
|------|------|
| 言語 | Rust |
| フレームワーク | Bevy (ECS + レンダリング + 入力) |
| 物理計算 | MVP: CPU O(N²) → 後フェーズで GPU compute (wgpu) |
| 力の定義 | Rust trait による関数ベースプラグイン |
| 数値積分 | Velocity Verlet (シンプレクティック、N体標準) |

## 計画概要

### Story 1: [基本シミュレーション](basic_simulation/overview.md)
1000体がニュートン重力で動くのを 3D で観る。最小動作する MVP。

### Story 2: [シミュレーション制御・パラメータ探索](simulation_control/overview.md)
一時停止・再開・新規開始、物理・初期条件の調整、距離多項式による力の式カスタマイズ。

### Story 3: [カメラ拡張](camera_enhancement/overview.md)
自由な視点操作（パン・ズーム・オービット・追従）。

### Story 4: [物体編集](object_editing/overview.md)
物体の追加・削除・位置変更。

### Story 5: [GPU Compute](gpu_compute/overview.md)
GPU compute shader による大規模化。10万体以上対応。

### Story 6: [Web 専用 GPU シミュレータ](wasm_web/overview.md)
Web 専用に再設計。Trunk + Bevy WebGPU で 10,000 体を GPU compute 駆動。GitHub Pages 公開。

## Story 間の依存関係

```
Story 1 (MVP)
  ├─→ Story 2 (シミュレーション制御)
  ├─→ Story 3 (カメラ拡張)
  ├─→ Story 4 (物体編集)
  └─→ Story 5 (GPU compute)
```

Story 1 完了後、Story 2〜5 は互いに独立して着手可能。Story 6 は Web 専用再設計のため Story 1〜5 と独立（既存デスクトップコードの互換は不要）。

## 受け入れ条件（プロジェクト全体）

- [ ] 1000体がリアルタイム (60fps) で動作する
- [ ] 3D ビジュアルでインタラクティブに観察できる
- [ ] 力の式をコード (Rust trait) で差し替え可能
- [ ] 再生・停止・早送りができる
- [ ] 物理・初期条件・力の式を UI から変更して銀河形成を観察できる
- [ ] 視点を自由に操作できる
- [ ] 物体を追加・削除・位置変更できる
