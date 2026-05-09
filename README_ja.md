# kotonoha-core

**Semantic Lineage System（SLS）のオープンソース・コア実装**を置く Kotonoha エコシステム向けリポジトリです。公開仕様に沿ったソースコード、開発者向け文書、実装上の手掛かりを置きます。

**英語版（主文書）:** [README.md](README.md)

## 範囲

- SLS のコアライブラリおよび実行時コンポーネント（Phase 2 以降）
- 開発者向けドキュメント、ビルド手順、コントリビューションに関する注記

規範となる公開仕様は [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) に置きます。コードと仕様セクションの対応は **[仕様トレース（英語）](docs/spec-traceability.md)** を参照してください。

### Phase 2（現状の最小構成）

Rust クレート **`kotonoha_core`**：`lineage::LineageUnit`、`rde::validate_json`、`interchange::validate_interchange_json`、`TARGET_SPEC_BUNDLE`（詳細は [README.md](README.md)）。

### ビルド

[Rust](https://www.rust-lang.org/tools/install) が必要です。

```bash
cargo test
```

### 永続化（デプロイメント）

本番整合の中心ストアは **PostgreSQL 一本**とする（詳細は英語 [`docs/persistence.md`](docs/persistence.md)）。

## 関連リポジトリ

公開リポジトリ同士の参照のみ記載します。

| リポジトリ | 役割 |
| --- | --- |
| [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) | 公開仕様の正本 |
| **kotonoha-core（本リポジトリ）** | SLS の OSS コア実装 |
| [`kotonoha-cli`](https://github.com/zyx-corporation/kotonoha-cli) | 公式 CLI（[`CLI 定義`](https://github.com/zyx-corporation/kotonoha-cli/blob/main/docs/cli-definition.md)） |
| [`kotonoha-docs`](https://github.com/zyx-corporation/kotonoha-docs) | 仕様に含まない公開ドキュメント（マニュアル・チュートリアル等） |

可能な限り、実装の変更は [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) の仕様と整合させます。設計論点が未解決の場合は、公開仕様の側で整理してからコードの振る舞いを固定します。

## 言語について

**`kotonoha-core` 以下の文書は、デフォルトでは英文とします。** 必要に応じて日本語版を英語版に併置します。英語を主文書とし、日本語ファイルには原則 `*_ja.md` サフィックスを付けます（例: `README.md` / `README_ja.md`）。

## ライセンス

特段の記載がない限り、リポジトリ内のコンテンツは [Apache License 2.0](LICENSE) に従います。

## リンク

- 本リポジトリ: https://github.com/zyx-corporation/kotonoha-core
