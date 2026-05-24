# 計測結果（2026-05-24）

## 計測環境

| 項目 | 値 |
|------|-----|
| マシン | Apple M5 / 32 GiB |
| OS | macOS 26.5 |
| GPU バックエンド | Metal (WebGPU) |
| ビルド | `release` |
| 設定 | デフォルト（active_count=10,000、Newtonian、time_scale=1.0） |
| 計測手段 | `RenderDiagnosticsPlugin` + Profiling オーバーレイ + `GRAVITIUM_PROFILE_DUMP=1` |

## 計測上の制約（重要）

Metal / WebGPU では Bevy の pass 別 **GPU ms が取得できない**（`—` 表示）。  
表示される **CPU ms はコマンド記録時間のみ**（各 pass 約 0.001 ms）で、実際の GPU 実行時間を反映しない。

**フレーム時間（Frame ms）と FPS が実効性能の指標**となる。

### Metal 代替手法（Phase 4）

| 手段 | 状態 |
|------|------|
| Bevy `RenderDiagnosticsPlugin` GPU ms | Metal では未対応 |
| wgpu timestamp query | WebGPU feature 依存のため未採用 |
| **Frame ms ベンチ（`GRAVITIUM_BENCH=1`）** | 実装済 — macOS/Metal で A/B 比較可能 |
| Vulkan/DX12 環境 | CI 未整備 — 将来 pass 別 GPU ms 取得用 |

## スナップショット（最適化前ベースライン）

### frame 180（起動約 11 秒、`GRAVITIUM_PROFILE_DUMP` ログ）

| 指標 | 値 |
|------|------|
| active bodies | 9,982 / 10,000（18 体 merge 済み） |
| FPS | 30.0 |
| Frame | 33.39 ms |
| Sim CPU total | 0.008 ms |
| Merge CPU total | 0.003 ms |
| main_opaque_pass_3d CPU | 0.013 ms |

### 定常状態（起動約 20 秒、オーバーレイ smoothed 値）

| 指標 | 値 |
|------|------|
| active bodies | 9,959 / 10,000（41 体 merge 済み） |
| FPS | **8.6** |
| Frame | **116.21 ms** |
| Sim CPU total | 0.004 ms |
| Merge CPU total | 0.001 ms |
| main_opaque_pass_3d CPU | 0.007 ms |

## 適用済み最適化（2026-05-24）

| Phase | 内容 |
|-------|------|
| 1 | `active_count` ベース dispatch、gravity `inverseSqrt`、merge `cbrt` 事前計算、bind group 条件付き更新、readback 範囲縮小 + 2 フレームに 1 回、pick を `active_count` に |
| 2 | merge find+apply を単一 compute pass に統合（6→5 pass 境界）、bucket 32,768、フレーム内 merge 2 反復 |
| 3 | 描画 mesh を `active_count` スロットのみ（10k × 42 頂点）、icosphere subdivision 2→1 |
| 4 | `GRAVITIUM_BENCH=1`、`GRAVITIUM_CHECKSUM=1`、物理状態ハッシュ |

## 解釈

1. **目標（60 fps / 16.7 ms）に対し約 7 倍遅い**（116 ms/frame、最適化前）。10k 体デフォルト設定では macOS 上で快適なリアルタイム性が出ていない。
2. **pass 別 CPU ms はほぼゼロ** → Metal 上ではボトルネック特定に使えない。116 ms の大半は **計測不能な GPU 実行 + 同期**。
3. **merge 中でも Merge CPU total は 0.001 ms 程度**。merge 6 パス自体はコマンド記録コストでは無視できる。ただし **GPU 側 merge コストは未計測**。
4. **merge 発生後も active bodies は 9,959 とほぼ 10k のまま** → この run では merge による n の減少効果は小さい。重さの主因は **依然として 10k 規模の全対全重力** と **描画** と推定。
5. **理論上の支配項**: gravity O(n²) ≈ 1 億ペア/フレーム。merge O(n)（クラスタ時悪化）。描画 active_count × 42 頂点 icosphere（最適化後）。

## ボトルネック推定（フレーム時間ベース）

| 領域 | 推定寄与 | 根拠 |
|------|----------|------|
| Gravity compute | **大** | O(n²)、n=10k、フレーム時間の大半を占める典型パターン |
| Bodies 描画 | **中〜大** | 単一 draw、最適化後 ~42 万頂点（10k × 42） |
| Merge compute | **小〜中** | 通常 O(n)。クラスタ時のみ増加 |
| Integrate / Colors | **小** | O(n) |
| CPU readback + bind group 再生成 | **小** | 条件付き bind group + 部分 readback で削減 |

## 再計測手順

### 手動（オーバーレイ + スナップショット）

```bash
cargo build --release
GRAVITIUM_PROFILE_DUMP=1 RUST_LOG=info target/release/gravitium
```

- frame 180 / 600 で stdout にスナップショット出力
- Profiling オーバーレイ（Display タブで ON/OFF）

### 自動ベンチマーク（Frame ms avg / p95）

```bash
cargo build --release
GRAVITIUM_BENCH=1 GRAVITIUM_BENCH_FRAMES=600 GRAVITIUM_BENCH_WARMUP=180 \
  RUST_LOG=info target/release/gravitium
```

stdout に JSON 1 行（例）:

```json
{"kind":"bench","seed":12345678,"active_count":10000,"frames":600,"warmup_frames":180,"frame_ms_avg":45.2,"frame_ms_p95":52.1}
```

終了後にプロセスは自動終了する。

### 物理回帰 checksum

```bash
cargo build --release
GRAVITIUM_CHECKSUM=1 GRAVITIUM_CHECKSUM_FRAMES=600 RUST_LOG=info target/release/gravitium
```

stdout に JSON 1 行（例）:

```json
{"kind":"checksum","seed":12345678,"active_count":10000,"frame":600,"hash":"a1b2c3d4e5f67890"}
```

最適化前後で同一 seed・同一設定なら hash が一致することを確認する。

### merge 密集シナリオ

再現手順（固定 seed 12345678）:

1. Physics タブで **Merge radius factor** を最大付近（100）に設定 → Apply
2. Initial タブで **N central stars** を 4、**Active count** を 20,000 に設定 → Restart
3. 600 フレーム以上実行後、`GRAVITIUM_BENCH=1` で Frame ms を計測
4. 通常 run（デフォルト 10k）と比較

merge 密集時は hash 一致は要求しない（merge 反復 2 回/フレームにより短時間 run では分布が僅かに変わる可能性あり）。通常設定での checksum 一致を回帰の主指標とする。

## 描画と compute のオーバーラップ（Phase 3 調査）

Bevy render graph 上、`SimulationComputeNode` は `CameraDriverLabel` の直前に配置されている。  
compute pass 完了後に opaque 3D pass が走るため、**同一フレーム内で compute→render は直列**。

Metal では pass 別 GPU 時間が取れないため、compute/render オーバーラップの有無は Frame ms 総量でのみ評価可能。  
現状の graph では意図的な async overlap は未実装（将来: compute を前フレームに pipelining する余地あり）。

## 未計測・今後必要な計測

- [ ] Vulkan/DX12 環境での GPU ms（pass 別内訳）
- [ ] Phase 1 適用後の A/B（同一 seed・同一 active_count）— `GRAVITIUM_BENCH=1` で実施
- [ ] merge 密集シナリオでの Frame ms 比較
- [ ] dev ビルド vs release ビルドの比較
