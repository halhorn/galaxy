# Phase 2: GPU シミュレーション本体

## 目的

10,000 体の N 体重力シミュレーションを GPU 上で完結させ、instanced 描画でブラウザに表示する。プロジェクトの核心フェーズ。

## 計画概要

### Task 1: GPU バッファ設計
シミュレーション状態は GPU 常駐。ECS には載せない。

- `positions` (vec4 × N)
- `velocities` (vec4 × N)
- `masses` (f32 × N)
- `accelerations` (vec4 × N)
- N = 10,000（固定。初版は可変長にしない）

Startup で CPU から初期条件（ディスク状分布）を 1 回アップロードし、以降は GPU 上で更新。

### Task 2: Compute パイプライン
Bevy の `RenderDevice` / `RenderQueue` を使用（独立 Instance なし）。

- **Pass 1 — 加速度**: 既存 `gravity.wgsl` をベースに O(N²) ペア和
- **Pass 2 — 積分**: Velocity Verlet（位置更新 → 加速度再計算 → 速度更新）を WGSL で実装
- RenderApp 上の render system または render graph node として dispatch
- 永続バッファ。毎フレーム `create_buffer` しない

Workgroup size 256、dispatch = ceil(N / 256)。

### Task 3: Instanced 描画
10,000 個の ECS エンティティは作らない。

- 球メッシュ 1 種 + instance buffer（position + mass → color/emissive）
- position バッファを描画側から参照（または可視化用に bind）
- エミッシブ + Bloom（WebGPU で問題があれば Bloom 無効化）

### Task 4: タイムステップ
- `FixedUpdate` または render schedule で dt を渡し、compute dispatch をトリガ
- `SimulationConfig` リソースで time_scale 管理（初版は固定 1.0 でも可）

### Task 5: 既存コードの整理
Phase 2 完了時点で不要コードを削除。

- `gpu_force.rs`（独立 wgpu + readback 型）→ 新 `simulation/` モジュールに置換
- `force.rs` CPU 実装、`ForceCalculator` trait
- `integrator.rs` CPU 積分
- `merger.rs`
- `main.rs` の 10,000 体 spawn ループ

## 計画詳細

**Bevy の役割**: キャンバス、入力、カメラ、RenderDevice 取得、instanced draw。物理の中心は ECS ではない。

**Readback 禁止**: 毎ステップ CPU へ positions を戻さない。カメラ操作・UI 以外は GPU 完結。

**ソフトニング**: 既存と同様 ε = 0.01。G = 4π²（AU, M☉, yr 単位系）。

**将来拡張（初版スコープ外）**:
- 衝突マージ（`merger.rs` 相当）
- Force trait による力の差し替え
- 物体数可変・ユーザー編集

## 受け入れ条件

- [ ] WebGPU 上で 10,000 体が重力相互作用して動く
- [ ] instanced 描画で全物体が表示される
- [ ] 30fps 以上（目標 60fps）
- [ ] 毎フレームの CPU readback がない（物理データ）
- [ ] 独立 `wgpu::Instance` / `pollster` がコードベースに存在しない
