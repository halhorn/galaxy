# URL 状態同期

## 目的

UI で変更したシミュレーション設定を URL フラグメント（`#v=1&…`）に反映し、リロード後も同じ設定を復元できるようにする。  
設定 URL をコピーして共有すれば、他者が同じ物理・初期条件・力の式でシミュレーションを開ける（シェアボタン UI は今回スコープ外）。

フォーマット・レイヤー分割は [fractalium](https://github.com/halhorn/fractalium) の `#<可読クエリ>` 方式を踏襲する。

## 現状

| 項目 | 状態 |
|------|------|
| 設定の永続化 | なし（リロードでデフォルトに戻る） |
| URL 連携 | なし |
| 対象プラットフォーム | Web（WASM）が主。符号化ロジックはネイティブでも単体テスト可能 |

## 同期対象（Applied 状態）

URL に載せるのは **GPU / 再起動に効いている Applied 値** のみ。Draft（`ControlPanelDraft`）は URL 復元時に Applied と揃える。

| グループ | ソース | UI の反映タイミング |
|----------|--------|---------------------|
| 物理 | `SimulationSettings.physics` | スライダー操作で即時 |
| 再生 | `SimulationConfig.time_scale`, `PlaybackState.mode` | 即時 |
| 初期条件 | `SimulationSettings.initial` | Apply & Restart 後 |
| 力の式 | `SimulationSettings.force` | Apply & Restart 後 |

**載せないもの**: `accumulated_sim_time`（実行時刻）、`fixed_dt`（内部定数）、カメラ姿勢、Draft の未 Apply 値。

## モジュール設計

責務分割・ファイル配置・依存ルールは [architecture.md](architecture.md) を参照。

要点:

- **`encoding/`** — ワイヤ形式のみ（ドメイン非依存）
- **`url/applied`** — 同期対象 DTO（Bevy Resource の写し）
- **`url/payload`** — DTO ↔ `#v=1&…` 本文
- **`url/sync`** — hydrate / flush の Bevy オーケストレーション
- **`ports/` + `platform/`** — ブラウザ `#` I/O（trait + WASM / native 具象）
- **`ui/` / `simulation/`** — URL を import しない（`Changed<>` 検知は sync 側）

## 計画概要

### Phase 1: [符号化とペイロード](phase1_encoding_and_payload.md)

`encoding/`、`url/applied`、`url/payload`。

### Phase 2: [同期プラグインと復元](phase2_sync_plugin.md)

`ports/`、`platform/`、`url/sync`、`bootstrap` 統合。

## 依存関係

```
Story 2 Phase 1〜4（UI・SimulationSettings が揃っていること）
  → Phase 1 → Phase 2
```

Story 6（WASM Web 公開）と並行可能。Phase 2 完了時点で GitHub Pages 上のリロード・URL 共有が機能する。

## 設計方針（確定）

| 項目 | 決定 |
|------|------|
| 載せ方 | `location.hash` = `#v=1&g=…&seed=…&term=…`（search ではなく fragment） |
| 版 | `v=1`。不一致版は復号拒否 |
| 未知キー | 復号時は無視（将来拡張用） |
| ブラウザ更新 | `history.replaceState`（失敗時 `location.hash` フォールバック） |
| ネイティブ | I/O は no-op。payload / codec は `#[cfg(test)]` で検証 |
| シェア UI | 今回なし（ユーザーがアドレスバーからコピー） |
| `from=share` search パラメータ | 今回なし（fractalium の Copy link 用。必要になったら Phase 3 で追加可） |

## 受け入れ条件（Feature 全体）

- [ ] UI で物理・時間倍率・再生/一時停止を変えると、アドレスバーの `#` が更新される
- [ ] 初期条件・力の式は Apply & Restart 確定後にのみ URL が更新される
- [ ] 設定入り URL を開き直すと、同じ物理・初期条件・力・時間倍率・一時停止状態で起動する
- [ ] 不正・未対応版の `#` は無視し、デフォルト設定で起動する（クラッシュしない）
- [ ] 同一設定への flush は連続で走らない（直前トークンと一致なら skip）
- [ ] payload の round-trip 単体テストが通る
