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

### Task 5: 衝突マージ（旧 `merger.rs` 相当）

デスクトップ版 `src/physics/merger.rs` と同じ**物理仕様**を Phase 2 に含める。実装は GPU バッファ向けに置き換えてよい（ECS の despawn / メッシュ差し替えは使わない）。

#### スケジュール

旧 `PhysicsPlugin` と同様、**1 物理ステップの積分が終わった直後**にマージを実行する。

```
position_step → gravity → velocity_step → merge
```

（旧: `FixedUpdate` で `velocity_verlet_step` の後に `merge_colliding_bodies` を `chain`）

#### 半径・衝突判定

| 項目 | 値 |
|------|-----|
| 半径 | `r = 0.5 × m^(1/3)`（AU。描画スケールと衝突判定で共通） |
| マージ閾値係数 | `MERGE_RADIUS_FACTOR = 0.2` |
| マージ距離 | 中心間距離 `d < (r_i + r_j) × MERGE_RADIUS_FACTOR` |

`MERGE_RADIUS_FACTOR = 1.0` なら表面接触、`0.2` なら合成半径の 20% まで近づいたら合体（旧実装どおり、やや深いめり込みでマージ）。

#### 合体時の状態更新（保存則）

ペア `(i, j)` がマージ条件を満たしたとき、**インデックスの小さい方 `i` を残し、`j` を吸収**する（旧実装と同じ向き）。

| 量 | 更新式 |
|----|--------|
| 質量 | `m_i ← m_i + m_j` |
| 速度 | `v_i ← (v_i m_i + v_j m_j) / (m_i + m_j)`（合体前の `m_i`, `m_j` で計算） |
| 位置 | `x_i ← (x_i m_i + x_j m_j) / (m_i + m_j)` |
| 加速度 | `a_i ← 0`（次ステップで重力パスが再計算） |

吸収された `j` はシミュレーションから除外する。

- **旧 (ECS)**: `j` のエンティティを `despawn`
- **Phase 2 (GPU)**: スロット `j` を非活性化（例: 質量 0、描画スキップ、重力ループで `j` をスキップ）。固定 N=10,000 のままコンパクションは初版では行わない

色・見た目の半径は質量から導出（`bodies.wgsl` の `radius_from_mass` / `color_from_mass` と一致）。マージ後は質量更新だけで表示が追従する。

#### ペア探索（性能）

旧実装と同様 **空間ハッシュ** で候補ペアを絞る（全ペア O(N²) は避ける）。

- セルサイズ: `(2 × max_radius × MERGE_RADIUS_FACTOR).max(0.01)`（全活性体の最大半径から算出）
- 各体をセルに登録し、同一セルおよび隣接 27 セル内のペアのみ距離判定
- 同一フレームで複数マージが起きうる。旧実装は `absorbed[]` で吸収済みをスキップし、`i` 側に連続マージしうる

#### GPU 実装方針（参考。Task 5 の受け入れには含めない）

コード構成は自由。例:

- **Compute パス** `merge.wgsl`: 空間グリッド構築 + 衝突解決（アトミックまたはソート済みペア）
- **CPU フォールバック**: 低頻度 readback + 旧アルゴリズム相当（初版プロトタイプ用）

いずれにせよ、**毎フレームの全 positions readback を恒常運用しない**方針（Phase 2 全体方針）は維持。マージ用の限定 readback は許容するか、完全 GPU 化するかは実装時に決定。

#### Task 5 受け入れ条件

- [ ] 二重星・ディスク粒子が近接すると品質・運動量保存則どおりに 1 体にまとまる
- [ ] マージ後の半径・色が質量 `m_i` から一意に決まる（`r = 0.5 m^(1/3)`）
- [ ] 吸収されたスロットは描画されず、重力計算にも寄与しない
- [ ] `MERGE_RADIUS_FACTOR = 0.2`、閾値式は上表と一致

### Task 6: 既存コードの整理
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

**将来拡張（Task 5 完了後もスコープ外）**:
- Force trait による力の差し替え
- 物体数可変・非活性スロットのコンパクション（マージ後も N 固定のまま）

## 受け入れ条件

- [ ] WebGPU 上で 10,000 体が重力相互作用して動く
- [ ] instanced 描画で全物体が表示される
- [ ] Task 5 の衝突マージ仕様を満たす
- [ ] 30fps 以上（目標 60fps）
- [ ] 毎フレームの CPU readback がない（物理データ。マージ用の限定 readback は可）
- [ ] 独立 `wgpu::Instance` / `pollster` がコードベースに存在しない
