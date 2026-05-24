# Phase 3: 描画最適化

## 目的

`main_opaque_pass_3d` および bodies mesh（20,000 × 92 頂点 ≈ 184 万頂点）の GPU 負荷を下げる。物理計算は不変、見た目は許容範囲の近似可。

## 背景

計測 run では Sim CPU total は微小だが、**icosphere 20k インスタンス描画**が Frame ms に中〜大の寄与と推定。非アクティブ body も VS で変換後 clip している。

## Task 1: 非アクティブ body の描画スキップ

**現状**: 全 20,000 スロット分の頂点を毎フレーム処理。

**作業案**（いずれか）:
- indirect draw でアクティブ body のみ dispatch
- mesh を active スロットのみに分割（restart 時再構築）
- 小質量星を point / billboard に切替

### 受け入れ条件

- [ ] inactive body の描画コストがゼロに近づく
- [ ] active body の見た目が許容範囲
- [ ] Frame ms が Phase 1 後比 **10% 以上改善**

---

## Task 2: icosphere subdivision 低減

**現状**: subdivision=2（92 頂点/body）。

**作業**: subdivision=1 等に下げ、視覚確認。

### 受け入れ条件

- [ ] 頂点数が半減以下
- [ ] ズーム時の品質が許容範囲

---

## Task 3: 描画と compute のオーバーラップ（調査）

**作業**: render graph 上で compute 完了待ちが Frame ms を押し上げていないか調査。

### 受け入れ条件

- [ ] 調査結果が measurements または architecture に記録されている

---

## Phase 3 完了条件

- [ ] Task 1 または Task 2 を適用
- [ ] 10k 体通常 run で **Frame ms ≤ 33 ms（30 fps 以上）** を目指す（Phase 1+3 合算）
