# Phase 1: 符号化とペイロード

## 目的

fractalium と同じ **フラット可読クエリ** 形式で、Gravitium の Applied 設定を文字列化・復元する。  
Bevy / WASM に依存しない層として `encoding/`、`url/applied`、`url/payload` に置く（[architecture.md](architecture.md)）。

## ワイヤフォーマット（fractalium 準拠）

フラグメント本文（`#` なし）の例:

```text
v=1&g=39.478&soft=0.01&merge=0.2&ts=1&pause=0&seed=1784561231&nstars=2&stmass=1&ror=3&dmmin=0.001&dmmax=0.01&drmin=5&drmax=50&dh=0.5&vpert=0.05&active=10000&term=s:1,e:-3,c:39.478
```

### トップレベル書式

- セグメントを `&` で連結。各セグメントは `キー=値`
- 復号時、値側のみ [`urlencoding`](https://crates.io/crates/urlencoding) でパーセント復号
- エンコード時は ASCII トークン前提でパーセントエンコードしない（fractalium と同じ）
- 同一キーの繰り返し（`term=` 複数）可

### サブレベル書式（`term=` の値など）

- `,` 区切りの値列、または `:` 区切りの `サブキー:値` ペア（fractalium `SubLevel` と同型）

## 配置（Phase 1 で追加するファイル）

```
src/encoding/mod.rs
src/encoding/flat_query_codec.rs
src/url/mod.rs
src/url/applied.rs
src/url/payload.rs
```

`lib.rs` に `mod encoding; mod url;` を追加。Phase 2 まで `ports` / `platform` / `sync` は未追加。

## 計画概要

### Task 1: `encoding/flat_query_codec`

fractalium [`flat_query_codec`](../../fractalium/src/encoding/flat_query_codec.rs) をベースに移植。

| 型 / 関数 | 責務 |
|-----------|------|
| `TopLevel` | `decode` / `encode` / `pairs` |
| `SubLevel` | `decode_to_f32` / `decode_to_u32` / `decode_to_kv_pairs` / `encode_from_kv_f32` 等 |
| `WIRE_F32_SIG_FIGS` | 浮動小数の有効桁（6 桁） |

**依存**: `urlencoding` のみ。`model` / Bevy 禁止。

### Task 2: `url/applied` — 同期 DTO

Bevy Resource を payload が直接触らないよう、Applied 状態のスナップショット型を置く。

| 型 / 関数 | 責務 |
|-----------|------|
| `AppliedUrlState` | `physics`, `initial`, `force`, `time_scale`, `paused` |
| `from_resources` / `apply_to_resources` | `SimulationSettings` + `SimulationConfig` + `PlaybackState` との変換 |
| `sync_draft` | `ControlPanelDraft` の initial / force を Applied に揃える |

**依存**: `model/`、`simulation::config` / `playback` の型のみ（Bevy Plugin / System 禁止）。

### Task 3: `url/payload` — キー定義と検証

定数 `URL_STATE_VERSION: u32 = 1`。

| キー | 型 | 対応フィールド |
|------|-----|----------------|
| `v` | u32 | フォーマット版 |
| `g` | f32 | `PhysicsSettings.g` |
| `soft` | f32 | `PhysicsSettings.softening` |
| `merge` | f32 | `PhysicsSettings.merge_radius_factor` |
| `ts` | f32 | `SimulationConfig.time_scale` |
| `pause` | u8 (0/1) | `PlaybackState.mode` |
| `seed` | u64 | `InitialConditions.seed` |
| `nstars` | u32 | `InitialConditions.n_stars` |
| `stmass` | f32 | `InitialConditions.star_mass` |
| `ror` | f32 | `InitialConditions.star_orbit_radius` |
| `dmmin` / `dmmax` | f32 | 円盤質量範囲 |
| `drmin` / `drmax` | f32 | 円盤半径 |
| `dh` | f32 | `InitialConditions.disk_height` |
| `vpert` | f32 | `InitialConditions.initial_v_perturbation` |
| `active` | u32 | `InitialConditions.active_count` |
| `term` (繰り返し) | `s,e,c` | `ForceTerm` 各項（sign, exponent, coefficient） |

公開 API（シグネチャレベル）:

| 関数 | 責務 |
|------|------|
| `encode_applied_state(&AppliedUrlState) -> Result<String, String>` | `#` なしクエリ本文を生成 |
| `decode_applied_state(&str) -> Result<AppliedUrlState, String>` | 復号 + 検証 + clamped |
| `validate_*`（内部） | 有限値・既存 `clamped()` 範囲・`ForceLaw::is_valid` |

**依存**: `encoding/`、`url/applied`、`model/` のみ。

### Task 4: 単体テスト

| テスト | 内容 |
|--------|------|
| round-trip | デフォルト設定 → encode → decode → 各フィールド一致 |
| round-trip | 力 2 項 + 初期条件カスタム |
| 未知キー無視 | `&future=1` を付けても復号成功 |
| 版不一致 | `v=2` で拒否 |
| 不正値 | 非有限 f32・空 force で `Err` |

## 計画詳細

- **デフォルト省略**: 初版は fractalium と同様、**全キーを常に出力**（実装単純・diff しやすい）。URL 短縮は将来 Task。
- **`pause`**: 共有時に「一時停止で開く」用途のため載せる。sim time は載せない。
- **`seed`**: 10 進文字列（`u64`）。hex は使わない（可読性優先）。
- **force の sign**: ワイヤ上 `s:1` / `s:-1`（内部 `ForceTerm.sign` と一致）。
- **依存**: `Cargo.toml` に `urlencoding = "2"` を追加。

## 受け入れ条件

- [ ] `encoding/` に Bevy / model / simulation の import がない
- [ ] `url/payload` に Bevy / web-sys の import がない
- [ ] encode/decode の入出力が `AppliedUrlState` に統一されている
- [ ] デフォルト Applied 状態の round-trip テストが通る
- [ ] 不正クエリが `Err` を返し、呼び出し側が無視できる
- [ ] 生成文字列が `v=1` で始まる
