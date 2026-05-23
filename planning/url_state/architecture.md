# モジュール設計（URL 状態同期）

## 目的

URL 同期を **ワイヤ形式・ドメイン変換・ブラウザ I/O・Bevy オーケストレーション** に分割する。  
既存の `model` / `simulation` / `ui` / `view` レイアウト（[simulation_control/architecture.md](../simulation_control/architecture.md)）を壊さず、横断 concern として `encoding` / `url` / `ports` / `platform` を追加する。

fractalium 対応:

| fractalium | gravitium |
|------------|-----------|
| `encoding/flat_query_codec` | 同名・同責務 |
| `app/share/payload` | `url/payload` |
| `app/share/sync` | `url/sync` |
| `ports/share_link` | `ports/url_fragment` |
| `platform/share_nav_*` | `platform/url_nav_*` |
| `app/platform_handles` | `url/navigation.rs`（単一 Resource のみ。専用 handles モジュールは不要） |

---

## レイヤー概要

```
┌─────────────────────────────────────────────────────────┐
│  ui/           パネル操作のみ。URL は触らない               │
└───────────────────────────┬─────────────────────────────┘
                            │ Resource 更新（既存どおり）
┌───────────────────────────▼─────────────────────────────┐
│  url/sync      hydrate / flush / PendingUrlSync           │
│                Bevy Resources ↔ AppliedUrlState           │
└───────┬───────────────────────────────┬─────────────────┘
        │ encode / decode                 │ read / write # 
┌───────▼──────────┐            ┌───────▼─────────────────┐
│  url/payload     │            │  ports/url_fragment      │
│  url/applied     │            │  （trait のみ）           │
└───────┬──────────┘            └───────┬─────────────────┘
        │ uses                          │ impl
┌───────▼──────────┐            ┌───────▼─────────────────┐
│  encoding/       │            │  platform/url_nav_*      │
│  flat_query_codec│            │  （WASM / native 具象）   │
└──────────────────┘            └──────────────────────────┘
        ▲
        │ 型のみ参照（Bevy 非依存）
┌───────┴──────────────────────────────────────────────────┐
│  model/          PhysicsSettings, InitialConditions, ForceLaw │
│  simulation/     SimulationConfig, PlaybackMode（型のみ）     │
└──────────────────────────────────────────────────────────────┘
```

**依存ルール**

| From | To | 禁止 |
|------|-----|------|
| `encoding/` | `urlencoding` crate | bevy, web-sys, model, simulation, ui, view |
| `url/payload`, `url/applied` | `encoding/`, `model/` | bevy, web-sys, ui, view, simulation/gpu |
| `url/applied` | `simulation::config`, `simulation::playback` の **型のみ** | Bevy Resource / Plugin / System |
| `url/sync` | `url/payload`, `ports/`, `simulation/`, `ui::draft` | view, simulation/gpu, encoding（payload 経由のみ） |
| `ports/` | （なし） | bevy, model, url, platform |
| `platform/` | `ports/` | bevy, model, url, ui, simulation |
| `bootstrap/` | `url::UrlSyncPlugin`, `platform::url_navigation_arc` | url 内部の payload / encoding 直接 import 禁止 |

`simulation/` と `ui/` は **url を import しない**（sync が `Changed<>` で Applied 変更を検知する）。

---

## ディレクトリ・ファイル一覧

```
src/
├── lib.rs                          # mod encoding, url, ports, platform を追加
│
├── encoding/                       # ★ ワイヤ形式（ドメイン非依存）
│   ├── mod.rs
│   └── flat_query_codec.rs         # TopLevel, SubLevel, decode/encode
│
├── url/                            # ★ URL 同期ドメイン + Bevy オーケストレーション
│   ├── mod.rs                      # pub use UrlSyncPlugin, AppliedUrlState
│   ├── applied.rs                  # AppliedUrlState（同期対象のスナップショット DTO）
│   ├── payload.rs                  # AppliedUrlState ↔ クエリ本文（v=1&…）
│   ├── navigation.rs               # UrlNavigation Resource（Arc<dyn UrlFragmentPort>）
│   └── sync.rs                     # UrlSyncPlugin, hydrate, flush, PendingUrlSync
│
├── ports/                          # ★ 環境 I/O の trait 境界
│   ├── mod.rs
│   └── url_fragment.rs             # UrlFragmentPort trait
│
├── platform/                       # ★ trait の具象（OS / ブラウザ）
│   ├── mod.rs                      # url_navigation_arc()
│   ├── url_nav_wasm.rs             # history.replaceState / location.hash
│   └── url_nav_native.rs           # no-op スタブ
│
├── bootstrap/mod.rs                # UrlSyncPlugin 登録、UrlNavigation 注入
│
├── model/                          # （変更なし）payload が参照する型の定義元
├── simulation/                     # （変更なし）sync が Resource を読む
├── ui/                             # （変更なし）URL 非依存
└── view/                           # （変更なし）URL 非依存
```

---

## ファイル別責務

### `encoding/flat_query_codec.rs`

**やること**: `key=value&…` のパースと組み立て。浮動小数の桁丸め。  
**やらないこと**: Gravitium のキー名・`SimulationSettings` の意味・`#` / `location` 操作。

公開: `TopLevel`, `SubLevel` と encode/decode ヘルパ（fractalium から移植）。

### `url/applied.rs`

**やること**: URL に載せる Applied 状態を 1 つの plain struct にまとめる。

| 型 | フィールド | 由来 |
|----|-----------|------|
| `AppliedUrlState` | `physics`, `initial`, `force`, `time_scale`, `paused` | 各 Resource の写し |

| 関数 | 責務 |
|------|------|
| `AppliedUrlState::from_resources(settings, config, playback)` | Bevy Resource → DTO |
| `AppliedUrlState::apply_to_resources(self, settings, config, playback)` | DTO → Resource 上書き |
| `AppliedUrlState::sync_draft(self, draft)` | DTO → `ControlPanelDraft`（initial + force） |

Bevy の `Resource` trait には impl しない。`sync.rs` だけが橋渡しする。

### `url/payload.rs`

**やること**: `AppliedUrlState` とクエリ本文（`#` なし）の相互変換、版チェック、検証、`clamped()` 適用。

| 関数 | 責務 |
|------|------|
| `encode_applied_state(state: &AppliedUrlState) -> Result<String, String>` | 本文生成 |
| `decode_applied_state(query: &str) -> Result<AppliedUrlState, String>` | 復号 + 検証 |

内部: キー定数（`URL_STATE_VERSION`）、`term=` 繰り返しの group 処理。  
**やらないこと**: Bevy System、`location` I/O、Draft / Restart の判断。

### `url/navigation.rs`

**やること**: `UrlNavigation(pub Arc<dyn UrlFragmentPort + Send + Sync>)` Resource 1 つ。  
bootstrap が `platform::url_navigation_arc()` の結果を insert する受け皿。

### `url/sync.rs`

**やること**: 起動 hydrate、変更 flush、dedupe、Plugin 登録。

| 型 / 関数 | 責務 |
|-----------|------|
| `UrlSyncPlugin` | Startup + PostUpdate システム登録 |
| `PendingUrlSync` | 次フレーム flush フラグ |
| `UrlHydrated` | URL 復元済み（初回 flush 抑制） |
| `hydrate_from_url` | Startup: fragment → decode → Resource + Draft |
| `detect_applied_changes` | `Changed<SimulationSettings>` 等 → `PendingUrlSync` |
| `flush_url_fragment` | encode → port.replace（WASM のみ実効） |

**やらないこと**: クエリキーのパース（payload へ委譲）、`web_sys` 直接呼び出し（port 経由）。

### `ports/url_fragment.rs`

**やること**: ブラウザ URL フラグメントの読み書き trait（fractalium `ShareNavigationPort` 相当）。

| メソッド | 責務 |
|----------|------|
| `current_fragment_body()` | `#` 以降（`#` なし）。なければ `None` |
| `replace_fragment_body(body)` | アドレスバー更新 |
| `fragment_equals(body)` | 無駄 replace 回避 |

今回スコープ外: `full_share_page_url`（Copy link 用。将来 `url/export.rs` 等で追加）。

### `platform/url_nav_wasm.rs` / `url_nav_native.rs`

**やること**: trait 具象のみ。WASM は `web_sys::Window` / `History`。native は no-op。  
**やらないこと**: 設定の encode/decode、Bevy。

---

## データフロー

### 起動（hydrate）

```
Startup (UrlSyncPlugin, SimulationPlugin より前)
  url/sync::hydrate_from_url
    → ports: current_fragment_body()
    → payload: decode_applied_state
    → applied: apply_to_resources + sync_draft
    → UrlHydrated = true

Startup (SimulationPlugin)
  spawn_initial_simulation  # 復元済み SimulationSettings を使用
```

### UI 変更 → URL（flush）

```
ui: SimulationSettings.physics 等を更新（既存）
  ↓
PostUpdate: detect_applied_changes (Changed<>)
  → PendingUrlSync = true
  ↓
PostUpdate: flush_url_fragment
  → applied::from_resources
  → payload::encode_applied_state
  → port::replace_fragment_body   # WASM のみ hash 更新
```

Apply & Restart も `SimulationSettings` / `PlaybackState` が変わるため、`Changed<>` で同じ経路。Draft 単独編集は Resource を触らないので flush されない。

### レイヤー越しの禁止例

| NG | 理由 |
|----|------|
| `ui/panels/*.rs` が `url::payload` を import | UI は Applied 更新のみ。URL は sync の責務 |
| `url/payload` が `bevy::prelude` を import | 単体テスト・ネイティブビルドの独立性 |
| `platform/` が `AppliedUrlState` を知る | I/O はバイト列（文字列）だけ扱う |
| `encoding/` が `model::ForceLaw` を知る | ワイヤ形式の再利用性 |

---

## Plugin 登録順（`bootstrap/mod.rs`）

```text
DefaultPlugins
  → UrlSyncPlugin          # hydrate Startup、PostUpdate flush
  → PanOrbitCameraPlugin
  → SimulationPlugin       # spawn_initial_simulation は hydrate 後
  → ViewPlugin
  → ControlUiPlugin
```

Startup チェーン（同一 Plugin 内）:

```text
hydrate_from_url  →  (SimulationPlugin) register_shaders → spawn_initial_simulation
```

`UrlSyncPlugin` と `SimulationPlugin` は別 Plugin のため、bootstrap 登録順で hydrate が先に走る。

PostUpdate チェーン:

```text
… ControlUiPlugin / SimulationRestartSet（Apply & Restart）
  → detect_applied_changes
  → flush_url_fragment
```

---

## 受け入れ条件（設計）

- [ ] `encoding/` に bevy / model / simulation の import がない
- [ ] `url/payload` に bevy / web-sys の import がない
- [ ] `simulation/` と `ui/` から `url/` を import していない
- [ ] `platform/` から `url/` / `model/` を import していない
- [ ] Applied 状態の encode/decode は `AppliedUrlState` 経由に統一されている
- [ ] ブラウザ I/O は `UrlFragmentPort` 実装以外に存在しない
