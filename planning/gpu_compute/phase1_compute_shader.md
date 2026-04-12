# Phase 1: GPU Compute Shader で力の計算を並列化

## 目的

O(N²) のペアワイズ力計算を GPU compute shader で実行し、10,000体以上でリアルタイム動作を実現する。

## 計画概要

### Task 1: wgpu セットアップ + WGSL シェーダ
- 独自の wgpu Device/Queue を作成（Bevy の render world とは独立）
- WGSL compute shader: 各スレッドが1体の加速度を計算（全 N 体をループ）
- バッファ: positions (vec4×N), masses (f32×N), accelerations (vec4×N), params (uniform)

### Task 2: GpuForceCalculator
- `ForceCalculator` trait を実装
- CPU→GPU バッファ書き込み → dispatch → staging buffer 経由で readback
- 最大 65,536 体まで対応（固定アロケーション）

### Task 3: PhysicsPlugin 統合
- `ActiveForce` リソースを GpuForceCalculator に差し替え
- GPU 初期化失敗時は CPU (NewtonianGravity) にフォールバック

## 設計メモ

- **Bevy render graph は使わない**: 物理計算はゲームロジック (FixedUpdate) で実行。Bevy のレンダリングパイプラインとは独立した wgpu device を使う。
- **同期実行**: dispatch → readback を毎ステップ同期で行う。非同期ダブルバッファリングは後の最適化。
- **Workgroup size**: 256 スレッド。dispatch = ceil(N/256)。
- **vec4 アライメント**: GPU は vec4 アクセスが最速。Vec3 を vec4 (w=0) にパックする。

## 受け入れ条件

- [ ] 10,000 体が 60fps で動作する
- [ ] GPU 初期化失敗時に CPU フォールバックが動作する
- [ ] 既存の物理テスト (2体軌道、エネルギー保存) が引き続き通る
