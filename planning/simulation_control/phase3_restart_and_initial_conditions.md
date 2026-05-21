# Phase 3: 新規開始 + 初期条件

## 目的

同じパラメータセットでやり直したり、星の数・円盤・物体数を変えて別の銀河形成を観察できるようにする。

## 計画概要

### Task 1: 初期条件リソース
- `model/initial.rs` — `InitialConditions`:
  - `seed: u64`
  - `n_stars: u32`（中心星数、1〜8 程度）
  - `star_mass`, `star_orbit_radius`
  - `disk_r_min`, `disk_r_max`, `disk_height`
  - `disk_radius_min`, `disk_radius_max`（質量分布）
  - `initial_v_perturbation`
  - `active_count: u32`（有効物体数、`n_stars + n_disk ≤ active_count ≤ BODY_COUNT`）
- 現行 `init.rs` のハードコード値を `InitialConditions::default()` に移行
- Applied 値は `SimulationSettings.initial`

### Task 2: 再初期化パイプライン
- `simulation/commands.rs` — `SimulationCommand::Restart`
- `simulation/restart.rs` + `simulation/upload.rs`:
  1. 一時停止
  2. `model::generate_initial_state(...)` → `BodyArrays`
  3. `active_count` 超過スロットは `mass = 0`（`≤ MIN_MASS`）で inactive 化
  4. `upload.rs` で GPU バッファ上書き
  5. `view/selection/snapshot.rs` — `ready = false`
  6. ユーザー操作で再開

### Task 3: 新規開始 UI
- `ui/panels/initial.rs` — ボタン「Apply & Restart」
- コントロール: シード（整数入力 + ランダムボタン）、中心星数、有効物体数、円盤半径範囲、擾乱強度
- 「Apply & Restart」: 初期条件確定 + Task 2 実行

### Task 4: active_count の GPU 反映
- `simulation/gpu/params.rs` — `GravityUniform.n` 等を `active_count` に
- シェーダループは既存の `params.n` 依存のため WGSL 変更不要（値の供給元のみ変更）

## 計画詳細

- **初期加速度**: 再起動時に CPU O(N²) で計算（現行 `compute_initial_accelerations` と同ロジック）。物体数 10,000 でも起動時一回のみ。
- **円盤粒子数**: `n_disk = active_count - n_stars`。`n_stars` 増加時は `active_count` も自動調整する UI バリデーション。
- **Selection / readback**: `SimulationCpuSnapshot` を再起動後に更新（既存選択機能との整合）。
- **RNG**: Phase 1 の `SimpleRng` を `seed` から生成。

## 受け入れ条件

- [ ] 「新規開始」でシミュレーション時間が 0 に戻り、初期配置から再スタートする
- [ ] シードを変えると異なるディスク配置になる（同シードで再現性あり）
- [ ] 中心星数を 1〜4 に変えて再開始できる
- [ ] 有効物体数を 2〜10,000 の範囲で変えて再開始できる
- [ ] 再開始後、inactive スロットの物体は描画・物理から除外される
- [ ] 再開始は一時停止状態で完了し、明示的に再開するまで進まない
