# Phase 2: 同期プラグインと復元

## 目的

Phase 1 のペイロードを WASM ブラウザの `#` と接続し、**起動時復元** と **UI 変更時の flush** を Bevy プラグインとして組み込む。  
モジュール配置の全体像は [architecture.md](architecture.md)。

## 配置（Phase 2 で追加するファイル）

```
src/ports/mod.rs
src/ports/url_fragment.rs
src/platform/mod.rs
src/platform/url_nav_wasm.rs
src/platform/url_nav_native.rs
src/url/navigation.rs
src/url/sync.rs          # Phase 1 の mod.rs から公開
```

`lib.rs` に `mod ports; mod platform;` を追加。`bootstrap/mod.rs` で Plugin 登録と `UrlNavigation` 注入。

## 計画概要

### Task 1: `ports/url_fragment` — I/O trait

fractalium [`ShareNavigationPort`](../../fractalium/src/ports/share_link.rs) に倣うが、Gravitium では **フラグメント操作のみ**（Copy link 用メソッドは今回なし）。

| メソッド | 責務 |
|----------|------|
| `current_fragment_body() -> Option<String>` | `#` 以降（`#` なし） |
| `replace_fragment_body(body) -> Result<(), String>` | `history.replaceState` / `set_hash` |
| `fragment_equals(body) -> bool` | 無駄な replace 回避 |

**依存**: なし（Bevy / model / url 禁止）。

### Task 2: `platform/url_nav_*` — trait 具象

| ファイル | 責務 |
|----------|------|
| `url_nav_wasm.rs` | `web_sys` で fragment 読み書き |
| `url_nav_native.rs` | no-op（`current` は常に `None`） |
| `mod.rs` | `url_navigation_arc() -> Arc<dyn UrlFragmentPort + Send + Sync>` |

`web-sys` features に `History`, `Location` を追加。

### Task 3: `url/navigation` + `url/sync` — Bevy 統合

fractalium [`SharePlugin`](../../fractalium/src/app/share/sync.rs) に倣う。符号化は payload、I/O は port に委譲。

| ファイル / 型 | 責務 |
|---------------|------|
| `navigation.rs` — `UrlNavigation` | `Arc<dyn UrlFragmentPort>` の Resource 受け皿 |
| `sync.rs` — `UrlSyncPlugin` | システム登録 |
| `PendingUrlSync` | 次フレーム flush フラグ |
| `UrlHydrated` | URL 復元済み（初回 flush 抑制） |
| `Local<Option<String>>` last_token | dedupe |

#### 起動: `hydrate_from_url`（Startup）

実行順: bootstrap 登録順により **`UrlSyncPlugin` → `SimulationPlugin`**（[architecture.md](architecture.md)）。

1. `port.current_fragment_body()`
2. `v=` で始まらなければ return
3. `payload::decode_applied_state` 成功時:
   - `AppliedUrlState::apply_to_resources`
   - `AppliedUrlState::sync_draft`
   - `UrlHydrated = true`
4. 失敗時はログのみ

再起動は不要 — 初回 spawn が復元済み settings を読む。

#### 変更検知: `detect_applied_changes`

PostUpdate で `Changed<SimulationSettings>` / `Changed<SimulationConfig>` / `Changed<PlaybackState>` を監視 → `PendingUrlSync = true`。

**ui / simulation は url を import しない**。Draft 単独編集は Resource を触らないため flush されない。Apply & Restart 後は `SimulationSettings` が変わるので同一経路。

#### flush: `flush_url_fragment`

`PendingUrlSync` が true のとき:

1. `AppliedUrlState::from_resources`
2. `payload::encode_applied_state`
3. `last_token` / `port.fragment_equals` で skip
4. `port.replace_fragment_body`
5. `last_token` 更新

`UrlHydrated` 直後の 1 フレームは flush しない。

### Task 4: `bootstrap` 統合

| 変更 | 内容 |
|------|------|
| `bootstrap/mod.rs` | `UrlSyncPlugin` を `SimulationPlugin` より前に `.add_plugins` |
| `bootstrap/mod.rs` | Startup で `UrlNavigation(platform::url_navigation_arc())` を insert |

### Task 5: 手動確認手順

1. デフォルト起動 → URL に `#v=1&…` が付く
2. G を変更 → hash が更新される
3. 初期条件 Apply & Restart → hash が更新される
4. カスタム hash 付き URL をリロード → 同設定・同初期配置（seed 一致）で起動
5. `#v=0` や壊れた hash → デフォルト起動

## 計画詳細

- **Draft と URL**: ユーザーが Draft だけ編集中は URL 不变。Apply 確定で Applied が変わり flush。
- **force パネル**: Apply 時に `settings.force` が更新される既存フロー（`process_pending_actions`）の **後** に flush フラグが立つこと。
- **履歴スタック**: `replaceState` のみ（戻るボタンで設定が巻き戻らない）。fractalium と同じ。
- **将来**: Copy link ボタン、`from=share` search、カメラ姿勢、sim time は別 Phase。

## 受け入れ条件

- [ ] WASM で UI 操作後、`location.hash` が `#v=1&…` 形式に更新される
- [ ] カスタム hash のリロードで設定が復元され、初回 spawn がその設定を使う
- [ ] 壊れた hash でクラッシュせずデフォルト起動する
- [ ] Draft 編集中（未 Apply）では hash が変わらない
- [ ] `ui/` と `simulation/` から `url/` を import していない
- [ ] `platform/` から `url/` / `model/` を import していない
- [ ] ネイティブビルドが URL 関連で link エラーなく通る
- [ ] `cargo test`（payload + codec）が通る
