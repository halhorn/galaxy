# Phase 3: デプロイ

## 目的

GitHub Pages に公開し、`/galaxy/` から誰でも WebGPU シミュレータにアクセスできるようにする。

## 計画概要

### Task 1: GitHub Actions
fractalium の `.github/workflows/deploy.yml` パターン。

- trigger: `main` push + `workflow_dispatch`
- `wasm32-unknown-unknown` target
- `jetli/trunk-action`
- `RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk build --release --public-url /galaxy/`
- Pages artifact アップロード → deploy

### Task 2: index.html 仕上げ
- `<title>`, description, viewport
- WebGPU 必須の説明文
- OGP / スクリーンショットは任意

### Task 3: README
- 公開 URL
- ローカル開発: `trunk serve` + `RUSTFLAGS`
- 対応ブラウザの記載

## 計画詳細

**public-url**: サブパス `/galaxy/` 必須。省略すると WASM/JS のパスが壊れる。

**参考**: `../fractalium/.github/workflows/deploy.yml`

## 受け入れ条件

- [ ] `main` push で CI が成功する
- [ ] `https://halhorn.github.io/galaxy/` から 10,000 体シミュレーションが起動する
- [ ] ローカル `trunk build --release --public-url /galaxy/` と CI 成果物が同等
