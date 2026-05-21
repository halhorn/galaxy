# Story 2: シミュレーション制御・パラメータ探索

## 目的

銀河形成の過程を観察・実験できるよう、ユーザーがシミュレーションを操作し、物理・初期条件・力の式を変えられる UI を追加する。  
現状は定数 (`constants.rs`) と起動時初期化 (`init.rs`) に固定され、一時停止や再開もできない。

## 現状（2025-05 時点）

| 項目 | 状態 |
|------|------|
| 時間制御 | なし（compute が毎フレーム無条件実行） |
| UI | カメラ操作のみ |
| 物理定数 | `G`, `SOFTENING`, `MERGE_RADIUS_FACTOR` 等はコンパイル時定数 |
| 初期条件 | 二重星 + 円盤、`init.rs` 内ハードコード |
| 力の式 | `gravity.wgsl` でニュートン重力（`∝ r / d³`）固定 |
| 物体数 | `BODY_COUNT = 10_000` 固定、全スロット使用 |

## 計画概要

### Phase 0: [モジュール分割リファクタ](phase0_refactor.md)
UI / 表示 / 純粋ロジック / GPU のレイヤー分離。詳細は [architecture.md](architecture.md)。

### Phase 1: [時間制御 + UI 基盤](phase1_time_control.md)
一時停止・再開・早送り。操作パネルの土台（bevy_egui）。

### Phase 2: [ランタイム物理パラメータ](phase2_physics_params.md)
重力定数 G、ソフトニング、衝突（マージ）距離、時間スケールを UI から変更し GPU uniform に反映。

### Phase 3: [新規開始 + 初期条件](phase3_restart_and_initial_conditions.md)
シード・中心星数・円盤形状・有効物体数などを変えて GPU バッファを再初期化。「新規開始」ボタン。

### Phase 4: [力の多項式](phase4_force_polynomial.md)
距離 d の整数乗の和（符号付き）で物体間の力を定義。例: `+d^-3`（ニュートン）, `-d^-2 + d^0` など。

## 依存関係

```
Phase 0 → Phase 1 → Phase 2 → Phase 3 → Phase 4
```

Phase 1 の一時停止は Phase 3（再初期化）・Phase 4（力変更）でも必須。Phase 2 と Phase 3 は順序入替可能だが、再初期化時に Phase 2 のパラメータを引き継ぐため Phase 2 を先に推奨。

## 設計方針（確定）

| 項目 | 決定 |
|------|------|
| UI | `bevy_egui`（Bevy / WASM 共通、canvas 上にオーバーレイ） |
| 物体数上限 | `BODY_COUNT` は固定。有効物体数 `active_count ≤ BODY_COUNT` のみ可変 |
| 力の多項式 | 最大 8 項。各項 `(sign, exponent, coefficient)` |
| パラメータ反映 | Main world の Resource → `ExtractResource` → Render world uniform |
| 力・初期条件の変更 | Apply 時に一時停止 → GPU 再計算（加速度リセット含む） |

## 未確定事項

[Question] 力の多項式で質量依存はどう扱うか（全項で `m_j` を掛ける現行と同じか、項ごとに変えるか）[/Question]  
[Draft] 現行重力と同様、各項に `masses[j]` を乗算。係数は無次元スカラーとする。[/Draft]
Answer: それで OK

[Question] 物体数スライダーの下限[/Question]  
[Draft] 100（マージ・描画の最低限。パフォーマンス検証後に調整可）[/Draft]
Answer: 2

## 受け入れ条件（Story 全体）

- [ ] 一時停止・再開・新規開始が UI とキーボード（Space）で操作できる
- [ ] G・ソフトニング・マージ距離・時間倍率を実行中に変更できる
- [ ] 星の数・円盤パラメータ・乱数シードを変えて新規開始できる
- [ ] 有効物体数を上限内で変更して再開始できる
- [ ] 符号付き距離多項式（整数指数）で力を定義し、Apply 後に挙動が変わる
- [ ] デフォルト設定（現行ニュートン重力 + 現行初期条件）で従来と同等の銀河風挙動が再現できる
