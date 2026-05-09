# kotonoha-core

**Semantic Lineage System（SLS）のオープンソース・コア実装**を置く Kotonoha エコシステム向けリポジトリです。公開仕様に沿ったソースコード、開発者向け文書、実装上の手掛かりを置きます。

**英語版（主文書）:** [README.md](README.md)

## 範囲

- SLS のコアライブラリおよび実行時コンポーネント
- 開発者向けドキュメント、ビルド手順、コントリビューションに関する注記（コードベースの成長に合わせて拡充）
- 仕様書とは切り分け、コードと一体で扱う実装上のメモ

規範となる公開仕様は [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) に置きます。本リポジトリは、それらの仕様に基づくコードと開発者向け成果物を中心に扱います。

## 関連リポジトリ

公開リポジトリ同士の参照のみ記載します。

| リポジトリ | 役割 |
| --- | --- |
| [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) | 公開仕様の正本 |
| **kotonoha-core（本リポジトリ）** | SLS の OSS コア実装 |

可能な限り、実装の変更は [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) の仕様と整合させます。設計論点が未解決の場合は、公開仕様の側で整理してからコードの振る舞いを固定します。

## 言語について

**`kotonoha-core` 以下の文書は、デフォルトでは英文とします。** 必要に応じて日本語版を英語版に併置します。英語を主文書とし、日本語ファイルには原則 `*_ja.md` サフィックスを付けます（例: `README.md` / `README_ja.md`）。

## ライセンス

特段の記載がない限り、リポジトリ内のコンテンツは [Apache License 2.0](LICENSE) に従います。

## リンク

- 本リポジトリ: https://github.com/zyx-corporation/kotonoha-core
