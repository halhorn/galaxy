# Phase 1: 低リスク・即効性

## 目的

挙動を変えずに、実装コストが小さい最適化で Frame ms を下げる。計測上 **Gravity + 固定 dispatch オーバーヘッド** が最大の改善余地。

## Task 1: active_count ベース dispatch

**現状**: `BODY_COUNT`（20,000）固定で workgroup dispatch。`active_count=10,000` 時、約半分のスレッドが idle return。

**作業**: integrate / gravity / merge / colors の dispatch 数を `active_count` 基準に変更。

**期待効果**: compute 系 GPU 時間 **~30–50% 削減**（active_count=10k 時）。

### 受け入れ条件

- [ ] `active_count=10,000` で dispatch workgroup 数が 79 → 40 前後になる
- [ ] `active_count=20,000` では従来と同数
- [ ] 同一 seed で 600 フレーム後の body 状態が変更前と一致

---

## Task 2: gravity シェーダ micro-opt

**現状**: 内側ループで `sqrt` + 除算。Newtonian fast path あり。

**作業**:
- `inverseSqrt` による距離計算の統一
- `term_count == 1` かつ exponent 別の specialize 強化

**期待効果**: gravity pass **5–15%** 改善（GPU 側、Metal では Frame ms で間接確認）。

### 受け入れ条件

- [ ] 単体テストまたは CPU `ForceLaw::compute_accelerations` との数値比較で誤差が許容範囲内
- [ ] Frame ms が Task 1 単独比で追加改善

---

## Task 3: merge `cbrt(mass)` 事前計算

**現状**: `mergeable()` が毎ペア `pow(mass, 1/3)` を呼ぶ（find_owner + apply_merge で二重）。

**作業**: `prepare` pass で scratch に物理半径（または cbrt）を書き込み、merge ホットループから `pow` 除去。

**期待効果**: merge 密集時に **Merge 関連 GPU 時間削減**。通常 run では Frame ms への寄与は限定的。

### 受け入れ条件

- [ ] merge 結果（survivor 索引・質量・位置）が変更前と一致
- [ ] merge 密集シナリオ（`merge_radius_factor` 高）で Frame ms が改善

---

## Task 4: bind group 条件付き更新

**現状**: 毎 render frame で 4 bind group + uniform を再生成。

**作業**: `SimulationSettings` / `SimulationConfig` が変更されたフレームのみ更新。

**期待効果**: CPU 側オーバーヘッド削減。Frame ms への直接効果は小さいが、UI 操作時の安定性向上。

### 受け入れ条件

- [ ] 設定変更なしの連続フレームで bind group が再生成されない
- [ ] physics スライダー変更後は正しく反映される

---

## Task 5: readback / pick スコープ縮小

**現状**: 毎フレーム 20,000 スロット readback。pick も `BODY_COUNT` 全走査。

**作業**:
- readback 範囲を `active_count` に限定、または 2–3 フレームに 1 回
- `pick_body_at_cursor` を `0..active_count` に

**期待効果**: CPU + PCIe 転送削減。ピッキング精度は 1–2 フレーム遅延可。

### 受け入れ条件

- [ ] クリック選択が機能する
- [ ] readback 転送量が半減以上（active_count=10k 時）

---

## Phase 1 完了条件

- [ ] Task 1–3 を適用
- [ ] [measurements.md](measurements.md) 手順で Frame ms が **116 ms → 80 ms 以下**（目安）
- [ ] 回帰: 同一 seed 600 フレームの checksum 一致
