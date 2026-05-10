# 単体／自動テスト指針（日本語）

**複製運用:** 社内向けドラフト **`24_unit_test_guidelines_ja.md`**（コア開発者も参照するワークスペース内の運用規程）と**同一の構成・見出し**で保守する。このファイルは **`kotonoha-core`** リポにおける転載であり、OSS クローンのみの読者にも意味が通じるように、関連リンクは公開リポジトリ中心とする。

---

## 本書の目的

**`kotonoha-core`** と周辺（**`kotonoha-cli`** 等）の **自動検証がどの層にあるか**、PR で何が期待されるかを、プロジェクト側で共通言語として揃える。normative は **`kotonoha-spec`** と開発フェーズ計画（クローン参加者の社内ワークスペースにおけるドラフト規程）により定められたフェーズゲートが優先する。

関連（公開のみ）: [`spec-traceability.md`](spec-traceability.md)、[`docs/tutorials/README.md`（英語チュートリアル索引）](https://github.com/zyx-corporation/kotonoha-docs/blob/main/docs/tutorials/README.md)、[`phase2_acceptance_demo.sh`](https://github.com/zyx-corporation/kotonoha-cli/blob/main/scripts/phase2_acceptance_demo.sh)、[`CONTRIBUTING.md`](../CONTRIBUTING.md)。**Awai 等の外部規程 URL はこの OSS ミラーでは張らない。** ワークスペース内の **`docs/24_*` / `25_*`** 転写は参加者の社内ドラフトが正本。

## 1. テストレイヤー（運用上の語彙）

| 層 | 何を証明するか | **`kotonoha-core` での当面の載せ場** |
| --- | --- | --- |
| **単体／ライブラリ** | 同一クレート内の型・関数（バリデータ等）が、期待どおり Accept／Reject／Warn すること | 各ソースファイル直下の **`#[cfg(test)] mod tests`**（Rust 標準） |
| **統合（DB・IO が絡む）** | DB へのマイグレーション〜永続〜読み返しなど、環境依存の経路が再現すること | Cargo の **`--features`** 付き `#[tokio::test]` 等。**外部 DB が要るときは `#[ignore]` と `DATABASE_URL` の明示運用**（[`CONTRIBUTING.md`](../CONTRIBUTING.md) と整合） |
| **受け入れ（フェーズゲート級）** | 「仕様に書いた最小要件」に対して、利用者視点で再現できる | Phase 2 は **`kotonoha`** CLI とスクリプト・CLI CI が運用上の証跡。手順：**Walkthrough**（上リンク）、自動化：**`phase2_acceptance_demo.sh`**（上リンク） |

**注意:** 「単体」を **関数１個＝仕様条文１個** と誤読しない。複数関数にまたがる**契約束**がある場合でも、環境依存を引かない検証なら単体側に載せうる。**仕様準拠そのものは `kotonoha-spec` と [`spec-traceability.md`](spec-traceability.md) が案内する。**

---

## 2. `kotonoha-core` の現状（2026-05-11 時点での観察）

| 項目 | 状況 |
| --- | --- |
| **`lineage`／`rde`／`interchange`** のライブラリテスト | 各 `*.rs` モジュール内に **計 30 本**（許容／拒否／strict・**エンベロープと lineage／RDE の形状 Negative** を中心） |
| **PostgreSQL 経路** | **`feature = postgres`** かつ **`#[ignore]`** 付き非同期テストが **計 2 本**。通常の `cargo test` では除外され、`DATABASE_URL` と `--include-ignored` が揃ったときのみ意味を持つ |
| **`tests/` ディレクトリ**による Rust インテグレーション | **未定義**（フェーズ検証経路として、将来コードハーネスを追加しうる） |

---

## 3. PR・CI における期待

1. **`cargo fmt --all --check`**、**`cargo test`**（および `cargo check --features postgres`）が **緑**。コア側 CI の方針に従う。
2. 公開規範に触れる振る舞いを変える場合、**[`spec-traceability.md`](spec-traceability.md)** に行または説明を足す／更新する（[`README.md`](../README.md) と同趣旨）。
3. **仕様のみで確定させる論点**は、`kotonoha-spec` 側の Issue／ escalation 手順と矛盾しない単位でテストへ落とす（テストだけで規範を勝手に拡張しない）。
4. **最小コード経路**を追加するときは **`cargo test` と Phase 2 受け入れスクリプトの後方整合**を明示確認する。

---

## 4. Positive と Negative／不変観（本リポジトリでの整理）

**Positive** は規範整合な受理・WARN・終了コードの回帰。**Negative** は `spec_bundle`／`spec_version` 不一致や禁止形状が混入しないことの断言。Flexible JSON が規範拡張のように見える状態を許さない（単体±Postgres で止める）。外部規程への直リンクは張らず、本節と **[`docs/unit_testing_guidelines.md`](unit_testing_guidelines.md)** を正とする。

## 5. Negative を厚くする領域の目安

ワークスペースにある運用ポリシードラフト **`19`**（トリガ）と強く結ぶ領域（規範、interchange、DDL、**`lost`** 等）ほど Negative を優先する。**`lost`** と **[#3](https://github.com/zyx-corporation/kotonoha-spec/issues/3)** および運用ドラフト **`08`** のトレースを切らない。

## 6. Red→Green→Refactor と PR

不変側は Negative を先に。複数論点なら分割。PR に「どの逸脱を防ぐか」を短文で。ΔM／ガード表現は **[`docs/unit_testing_guidelines.md`](unit_testing_guidelines.md)** およびワークスペースの **`25_*` 転写**に合わせる（OSS ミラーに私有 URL は出さない）。

## 7. CI と CLI フェーズ級

単体 rejects を CI で常時。Postgres は `DATABASE_URL`。利用者側の証跡は **`phase2_acceptance_demo`**（上リンク）を代替しない。**ブラウザ E2E**は現行 OSS ゲートには含めない。

---

## 変更履歴

| 日付 | 変更内容 |
| --- | --- |
| 2026-05-11 | 初版転載 |
| 2026-05-11 | §4〜7：Awai テスト／開発ドラフトおよび社内 **`24`/`25`** と整合 |
| 2026-05-11 | ライブラリ単体：**計 30 本**（lineage／interchange で **未知フィールド拒否**、`deny_unknown_fields` 込み）。テスト指針と整合 |