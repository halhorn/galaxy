# Galaxy — 重力シミュレーションプロジェクト

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

### Story 2: [シミュレーション制御](simulation_control/overview.md)
再生・停止・早送りの時間制御。

### Story 3: [カメラ拡張](camera_enhancement/overview.md)
自由な視点操作（パン・ズーム・オービット・追従）。

### Story 4: [物体編集](object_editing/overview.md)
物体の追加・削除・位置変更。

### Story 5: [GPU Compute](gpu_compute/overview.md)
GPU compute shader による大規模化。10万体以上対応。

## Story 間の依存関係

```
Story 1 (MVP)
  ├─→ Story 2 (シミュレーション制御)
  ├─→ Story 3 (カメラ拡張)
  ├─→ Story 4 (物体編集)
  └─→ Story 5 (GPU compute)
```

Story 1 完了後、Story 2〜5 は互いに独立して着手可能。

## 受け入れ条件（プロジェクト全体）

- [ ] 1000体がリアルタイム (60fps) で動作する
- [ ] 3D ビジュアルでインタラクティブに観察できる
- [ ] 力の式をコード (Rust trait) で差し替え可能
- [ ] 再生・停止・早送りができる
- [ ] 視点を自由に操作できる
- [ ] 物体を追加・削除・位置変更できる
