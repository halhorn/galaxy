# Phase 2: Merge パス最適化

## 目的

merge 発生時・クラスタ形成時の GPU 負荷を下げる。Phase 1 計測では通常 run では Merge CPU total は 0.001 ms と小さいが、**GPU 側は未計測**であり、密集時に frame 時間が跳ねる報告がある。

## 前提

Phase 1 完了後に着手。merge 密集シナリオでの再計測が必要。

## Task 1: merge 密集ベンチマークシナリオ定義

**作業**: 再現可能な merge 負荷シナリオを固定 seed で定義する。

候補:
- `merge_radius_factor` を最大付近
- `n_stars` を増やし中心クラスタを作る
- 長時間実行（600+ フレーム）後の計測

### 受け入れ条件

- [ ] シナリオ手順が [measurements.md](measurements.md) に追記されている
- [ ] 通常 run と merge 密集 run で Frame ms の差が計測できる

---

## Task 2: find_owner + apply_merge 融合の検討

**現状**: 同一 27 セル × リンクリスト走査を 2 回実行。`mergeable()` も二重呼び出し。

**作業**: 1 pass に統合できるか設計・検証。並列安全性（atomicMin + absorb の順序）を維持。

**期待効果**: merge pass 数 **6 → 5**、走査回数半減。

### 受け入れ条件

- [ ] 同一 seed・同一入力で merge 結果が融合前後で一致
- [ ] merge 密集シナリオで Frame ms 改善

---

## Task 3: hash bucket 数チューニング

**現状**: `MERGE_BUCKET_COUNT = 16,384` 固定。クラスタ時にチェーンが長くなる。

**作業**: 32,768 / 65,536 を試し、clear コスト vs 走査コストの trade-off を計測。

### 受け入れ条件

- [ ] 通常 run で Frame ms が悪化しない
- [ ] merge 密集 run で Frame ms が改善

---

## Task 4: フレーム内 merge 反復（任意）

**現状**: 1 フレーム 1 回 merge → 連鎖 merge は複数フレームに分散。

**作業**: 同一フレーム内で prepare→apply を 2 回実行。dt が小さいため挙動差は限定的と想定。

**期待効果**: 「merge 中の重い状態」の継続時間短縮（体感改善）。

### 受け入れ条件

- [ ] 短時間 run で total merge 回数・最終 body 分布が許容範囲
- [ ] merge 中の Frame ms ピークが低下

---

## Phase 2 完了条件

- [ ] merge 密集シナリオで Phase 1 完了時比 **Frame ms 10% 以上改善**
- [ ] 通常シナリオで物理結果の回帰なし
