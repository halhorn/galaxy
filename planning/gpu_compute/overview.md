# Story 5: GPU Compute

## 目的

GPU compute shader で力の計算を並列化し、10万体以上の大規模シミュレーションに対応する。

## 計画概要

### Phase 1: Compute Shader 実装
wgpu compute shader (WGSL) でペアワイズ力の計算を GPU 上で実行。Bevy の render graph 経由で統合。

### Phase 2: CPU/GPU 切り替え・フォールバック
実行時に CPU/GPU を切り替え可能に。GPU 非対応環境では CPU フォールバック。

## 受け入れ条件

- [ ] GPU compute で 10万体以上が 30fps で動作する
- [ ] CPU フォールバックが動作する
- [ ] Force trait の GPU 対応 (WGSL シェーダ生成 or パラメータ渡し)
