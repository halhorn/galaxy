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

- セルサイズ: `(2 × max_radius × MERGE_RADIUS_FACTOR).max(0.01)`（仕様）。実装は `max_radius` の代わりに `MERGE_MAX_RADIUS` 定数を使用（高速化メモ参照）
- 各体をセルに登録し、同一セルおよび隣接 27 セル内のペアのみ距離判定
- 同一フレームで複数マージが起きうる。旧実装は `absorbed[]` で吸収済みをスキップし、`i` 側に連続マージしうる

#### GPU 実装（`assets/shaders/merge.wgsl`）

完全 GPU 化。CPU readback なし。`src/simulation/compute.rs` の render graph node が積分後にマージパスを dispatch する。

#### 実装: マージ高速化

マージ追加後に FPS が大きく落ちたため、以下を実装した。

##### 問題と対策の経緯

| 問題 | 原因 | 対策 |
|------|------|------|
| 極端に遅い | 初版マージが `@workgroup_size(1)` で 10,000 体を 1 スレッド直列処理 | 並列 compute パスへ分割 |
| 真っ黒画面 | マージ bind group の storage buffer が **13 個**（WebGPU 上限 **8**/stage） | バッファ統合で **8 個**に削減 |
| 画面ビカビカ・マージ停止 | 2 フレームに 1 回だけマージする間引き | **毎フレームマージ**に戻す（採用しない） |
| マージが効かない | 固定セルサイズが小さすぎ（`max_radius ≈ 0.5` のみ想定） | `MERGE_MAX_RADIUS = 2.0` で保守的にセル拡大 |

##### 空間ハッシュ（旧 `merger.rs` と同趣旨）

全ペア O(N²) は使わない。バケット化して候補を絞る。

1. **セルサイズ**（初版は毎フレーム `max_radius` 走査、現行は定数で代替）:
   - `cell_size = max(2 × MERGE_MAX_RADIUS × MERGE_RADIUS_FACTOR, 0.01)`
   - 定数: `MERGE_MAX_RADIUS = 2.0`、`MERGE_RADIUS_FACTOR = 0.2` → `cell_size = 0.8` AU
   - 合体後の大きい星も隣接 27 セルに入るよう、半径上限を保守的に取る
2. **ハッシュ**: 3D 座標を `floor(pos × inv_cell_size)` でセル化し、16384 バケットへオープンアドレス（`hash % MERGE_BUCKET_COUNT`）
3. **登録**: 各活性体を `bucket_heads` + `bucket_next` の侵入リストに **並列**挿入（`atomicCompareExchange`）
4. **候補ペア**: 各体 `i` が自セルと **隣接 27 セル**内のリストだけ走査
5. **優先順**: `find_owner` で `atomicMin(merge_owner[j], i)`（`j > i` のみ）。最小 `i` が `j` を吸収 — 旧実装のループ順と同等
6. **適用**: `apply_merge` で `merge_owner[j] == i` のペアのみ `absorb`（同一スレッド内で `i` が複数 `j` を連続吸収可）

**GPU の条件分岐について**: 27 セル × リスト走査程度の分岐はコストより、探索範囲削減の利益が大きい。ボトルネックは依然 **O(N²) 重力** が第一候補。

##### 並列マージパイプライン（毎フレーム）

いずれも `@workgroup_size(256)`、`dispatch = ceil(N / 256)`（`clear_buckets` のみ `ceil(16384 / 256)`）。

```
prepare → clear_buckets → init_owner → build_grid → find_owner → apply_merge
```

| エントリ | 役割 |
|----------|------|
| `prepare` | `absorbed` クリア、スナップショット取得、`bucket_next` 初期化 |
| `clear_buckets` | `bucket_heads ← INVALID` |
| `init_owner` | `merge_owner[j] ← n`（無効オーナー） |
| `build_grid` | 空間ハッシュへ並列 insert |
| `find_owner` | 衝突候補に `atomicMin` でオーナー決定 |
| `apply_merge` | オーナー一致ペアを合体 |

スナップショットはパス開始時の位置・速度・質量のみ使用（距離判定・保存則。旧 `bodies[]` スナップショットと同じ）。

##### WebGPU storage buffer 上限への対応

ブラウザ WebGPU は compute stage あたり storage buffer **最大 8 個**（仕様デフォルト。設計も 8 上限）。マージ用に以下へ統合。

| バッファ | 内容 |
|----------|------|
| `positions`, `velocities`, `masses`, `accelerations` | シミュレーション本体（読み書き） |
| `merge_scratch` | `[0..n)` = `vec4(pos.xyz, mass)`、`[n..2n)` = velocity |
| `merge_bucket_heads` | `atomic<u32>` × 16384 |
| `merge_aux` | `[0..n)` = `bucket_next`、`[n..2n)` = `absorbed` |
| `merge_owner` | `atomic<u32>` × n |

計 8 個 + uniform（`MergeParams`: `n`, `merge_radius_factor`, `inv_cell_size`, `min_mass`）。

##### 非活性スロット（マージ後）

`masses[j] ← 0`（`MIN_MASS = 1e-8` 以下）とし、他パスでスキップ。

- `gravity.wgsl` / `integrate.wgsl`: `mass <= min_mass` なら early return
- `bodies.wgsl`: 非活性はクリップ外へ飛ばして描画しない

##### 採用しなかった案

- **マージのフレーム間引き**（例: 2 フレームに 1 回）: 見た目がチラつき、マージが進まない
- **CPU readback + 旧 `merger.rs`**: GPU 完結方針に反する
- **毎フレーム GPU 上で `max_radius` 全走査 + 動的 `inv_cell`**: 追加 storage buffer で上限超過。定数 `MERGE_MAX_RADIUS` で代替

##### 今後の高速化候補（未実装）

- 重力 `gravity.wgsl` のタイル化 / 近傍リスト（現状 O(N²) が支配的）
- 非活性スロットのコンパクション（N 可変化が必要）
- 毎フレームの `max_radius` reduction を **1 storage 追加**以内で行い、セルサイズを動的化

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
