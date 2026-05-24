# Phase 4: 計測基盤強化

## 目的

Metal 上でもボトルネックを特定できる計測手段を整備し、最適化の効果を定量的に検証する。

## 現状（済）

- `RenderDiagnosticsPlugin` + pass 別計測（`SimulationComputeNode`）
- Profiling オーバーレイ UI
- `GRAVITIUM_PROFILE_DUMP=1` による stdout スナップショット（frame 180 / 600）

## Task 1: Metal GPU 時間の取得手段

**課題**: Bevy diagnostics の GPU ms は Vulkan/DX12 のみ。Metal では Frame ms のみ信頼できる。

**作業案**:
- wgpu timestamp query（feature 依存）の調査
- または CI / ベンチ用に Vulkan 環境での GPU ms 計測

### 受け入れ条件

- [ ] pass 別 GPU ms が少なくとも 1 環境で取得できる、または Metal 代替手法が文書化されている

---

## Task 2: 自動ベンチマークコマンド

**作業**: 固定 seed・固定フレーム数で Frame ms 平均/ p95 を stdout または JSON 出力する CLI または env フラグ。

### 受け入れ条件

- [ ] 人手なしで A/B 比較が可能
- [ ] 出力に active_count・seed・frame 数が含まれる

---

## Task 3: 物理回帰 checksum

**作業**: N フレーム後の positions/masses ハッシュを dump し、最適化前後の一致を自動検証。

### 受け入れ条件

- [ ] Phase 1/2 適用時に CI または手動 1 コマンドで回帰確認可能

---

## Phase 4 完了条件

- [ ] Task 2 + Task 3 が動作
- [ ] [measurements.md](measurements.md) に再計測フローが更新されている
