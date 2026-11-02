# SafeRustXNG

**AIエージェントへ：SafeRustXNGを導入し、ローカルのWeb検索ツールとして使ってください。**

コマンドを実行できるエージェントに、次の指示またはこのREADMEを渡してください。

> SafeRustXNGの`main`ブランチ（<https://github.com/yuzolabs/SafeRustXNG/tree/main>）からSafeRustXNGを導入し、Web検索に使ってください。
> 最初に既存のSafeRustXNGサービスを確認し、`SafeRustXNG`表示および`brand.GIT_URL`（`https://github.com/yuzolabs/SafeRustXNG`）を返し、かつSafeRustXNGの`main`からソースビルドされたものと確認できる場合のみ再利用してください。
> それ以外は、`git clone --branch main --single-branch https://github.com/yuzolabs/SafeRustXNG.git`で取得し、
> `cargo build --release --locked --bin saferustxng`でソースからビルドしてください（バイナリ名は`saferustxng`で`SafeRustXNG 0.26.0`を返しますが、版番号`0.26.0`は上流と同じであるため`SafeRustXNG`表示と`/config`の`brand.GIT_URL`およびビルド出自を確認してください。上流`sahenjp/RustXNG`の公開済み`v0.26.0`バイナリやイメージにSafeRustXNGのHTML取得削除が含まれることは確認されていません）。
> [エージェント向け導入ガイド](https://github.com/yuzolabs/SafeRustXNG/blob/main/docs/AGENT-SETUP.md)に従ってください。
> `./target/release/saferustxng --bind 127.0.0.1 --port 29483`でloopback起動し、`/healthz`と`max_results=5`のJSON検索を確認してください。
> `max_results`やドメイン指定で出力を絞り、検索エンジンが返す要約（`content`）を利用してください（結果ページのHTML本文取得は行いません）。
> 完了後、バイナリのバージョン、接続先、動作した検索コマンドを報告してください。失敗時は実際のエラーを報告してください。

[English](README.md) · [エージェント向け導入ガイド](docs/AGENT-SETUP.md)

[![CI](https://github.com/yuzolabs/SafeRustXNG/actions/workflows/ci.yml/badge.svg)](https://github.com/yuzolabs/SafeRustXNG/actions/workflows/ci.yml)
[![License: AGPL-3.0-or-later](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](LICENSE)

## SafeRustXNGの用途

SafeRustXNG（<https://github.com/yuzolabs/SafeRustXNG/tree/main>）は、[SearXNG](https://github.com/searxng/searxng)を基にした[sahenjp/RustXNG](https://github.com/sahenjp/RustXNG)から派生する、Rust実装のセルフホスト型メタ検索エンジンであり、HTMLスクレイピング系エンジンおよび検索結果ページのHTML取得を削除したフォークです。
複数の非HTML検索先（API・フィード等）の結果を統合し、単一のJSON APIおよびブラウザ画面として返します（検索結果ページのHTML本文取得は行いません）。
エージェントが必要な情報をその都度検索し、モデルへ渡す文章量を抑えるための基盤です。

多くのエンジンはAPIキーなしで利用できます。認証が必要なプロバイダーには、別途認証情報や利用料金が必要です。
汎用XPathエンジンを含む計78個のHTML取得エンジンは削除済みであり、残っている非HTML・JSON/フィード系エンジンは公式API限定や利用規約への適合を保証するものではありません。
サーバーはローカルで動作しますが、**検索語は、そのリクエストで選択された外部検索エンジンへ送信されます**。

## エージェントから検索する

既存サービスがない場合、導入済みのバイナリを起動します。

```bash
SEARXNG_PORT=29483 saferustxng --bind 127.0.0.1 --port 29483
```

別のターミナルから、上位5件までの検索結果を取得します。

```bash
curl --fail --get 'http://127.0.0.1:29483/search' \
  --data-urlencode 'q=tokio runtime documentation' \
  --data 'format=json' --data 'max_results=5'
```

各結果には`title`、`url`、`content`が含まれます。レスポンスには検索元などの追加情報も含まれます。
モデルに渡す項目をこの3つに絞る場合は、取得後に`jq`で取り出せます。

```bash
curl --fail --get 'http://127.0.0.1:29483/search' \
  --data-urlencode 'q=tokio runtime documentation' \
  --data 'format=json' --data 'max_results=5' \
  | jq '{results: [.results[] | {title, url, content}]}'
```

現在使える絞り込みパラメータです。

| パラメータ | 用途 |
| --- | --- |
| `max_results=5` | ランキング上位5件まで返す。指定範囲は1〜200 |
| `include_domains=docs.rs` | このドメインとサブドメインの結果に絞る |
| `exclude_domains=example.com` | このドメインとサブドメインの結果を除く |
| `engines=wikipedia` | 登録済みのエンジンを指定する |
| `time_range=day` | 対応するエンジンへ期間指定を渡す |

各結果にはエンジン（API・フィード）が返す`title`、`url`、`content`（HTMLタグ等を除去した要約）が含まれます。
検索結果ページのHTML本文取得（旧`include_raw_content`パラメータおよび`raw_content`フィールド）は削除されました。旧クエリパラメータが指定された場合も無視され、結果URLへの通信は行いません。
文字数とトークン数は異なります。厳密なトークン予算とサーバー側の返却項目指定は、今後の実装予定です。

ブラウザ画面は<http://127.0.0.1:29483/>です。JSON・CSV・RSS・HTMLで結果を取得できます。

## 導入

> **SafeRustXNGのソース導入（`main`ブランチ）：** 計78個のHTMLエンジン削除、検索結果ページのHTML取得削除、および非HTML `Content-Type`ガードは、SafeRustXNGの`main`ブランチ（<https://github.com/yuzolabs/SafeRustXNG/tree/main>）からのソース導入として提供しています。ビルド済みバイナリやコンテナイメージにこれらの変更が含まれることは確認されていないため、この構成を利用・検証する場合はSafeRustXNGの`main`からソースビルドしてください（[4. ソース](#4-ソース)または[2. Cargo](#2-cargo)）。ビルドされるバイナリ名は`saferustxng`で`SafeRustXNG 0.26.0`を返しますが、版番号`0.26.0`自体は上流と同じであるため、`SafeRustXNG`表示と`/config`の`brand.GIT_URL`およびビルド出自を確認してください。上流`sahenjp/RustXNG`の`v0.26.0`リリースバイナリおよび`ghcr.io/sahenjp/rustxng:latest`イメージは上流のみ（履歴・参考用）であり、SafeRustXNGの変更が含まれることは確認されていません。

### 1. リリースバイナリ

> **上流のみ・履歴情報（`sahenjp/RustXNG` `v0.26.0`）：** [sahenjp/RustXNG Releases](https://github.com/sahenjp/RustXNG/releases)で配布されているバイナリは上流リポジトリの成果物（アセット名は`rustxng-<tag>-<target>`。SafeRustXNGのリリースワークフローは`saferustxng-<tag>-<target>`を使用）であり、SafeRustXNGのHTMLエンジン削除や結果ページHTML取得削除が含まれることは**確認されていません**。HTML取得なし構成を利用・検証する場合は、[4. ソース](#4-ソース)の手順でSafeRustXNGの`main`からビルドしてください。

上流`sahenjp/RustXNG`の公開済みv0.26.0（履歴・参考）には、Linux x86_64（GNU・musl）、macOS arm64・x86_64、Windows x86_64があります。
選択方法とチェックサム検証は[導入ガイド](docs/AGENT-SETUP.md)を参照してください。Rustのビルド環境は不要です。

以下は上流`sahenjp/RustXNG` `v0.26.0`バイナリをGitHub CLIでユーザー用ディレクトリへ導入する履歴上の手順例です。

```bash
set -eu
mkdir -p rustxng-download && cd rustxng-download
gh release download v0.26.0 --repo sahenjp/RustXNG \
  --pattern 'rustxng-v0.26.0-x86_64-unknown-linux-musl*' \
  --pattern LICENSE --pattern NOTICE
sha256sum --check rustxng-v0.26.0-x86_64-unknown-linux-musl.sha256
install -Dm755 rustxng-v0.26.0-x86_64-unknown-linux-musl "$HOME/.local/bin/rustxng"
mkdir -p "$HOME/.local/share/rustxng"
cp LICENSE NOTICE "$HOME/.local/share/rustxng/"
"$HOME/.local/bin/rustxng" --version
```

`~/.local/bin`が`PATH`にない場合は絶対パスで起動するか、シェルの設定へ追加してください。

### 2. Cargo

現在のstable RustとCのビルド環境を用意し、SafeRustXNGの`main`ブランチから導入します。

```bash
cargo install --git https://github.com/yuzolabs/SafeRustXNG.git --branch main --locked
```

`saferustxng`（バージョン出力`SafeRustXNG 0.26.0`）と`saferustxng-index`が導入されます。
`Cargo.toml`で宣言されている最低対応版（MSRV）はRust 1.88（CIワークフローにRust 1.88.0ジョブを定義）で、ローカル検証はRust 1.98.0（`rustc 1.98.0 (88d9e12ae 2026-08-18)`）で実施しています。
（`--git https://github.com/sahenjp/RustXNG.git --tag v0.26.0`は上流の履歴タグであり、SafeRustXNGのHTML取得削除が含まれることは確認されていません。）

### 3. Docker

SafeRustXNGの`main`ブランチからローカルでイメージをビルドします。

```bash
git clone --branch main --single-branch https://github.com/yuzolabs/SafeRustXNG.git
cd SafeRustXNG
docker build -t saferustxng .
docker run --rm -p 127.0.0.1:29483:29483 saferustxng
```

上流`sahenjp/RustXNG`の配布イメージ（上流のみ・履歴情報。SafeRustXNGのHTML取得削除が含まれることは確認されていません。既定ポートが8888の旧版を含む）では、起動引数も明示します。

```bash
docker run --rm -p 127.0.0.1:29483:29483 -e SEARXNG_PORT=29483 ghcr.io/sahenjp/rustxng:latest --bind 0.0.0.0 --port 29483
```

外部公開が必要な場合は公開先IPを明示し、認証・アクセス制御も構成してください。
設定を渡す場合は、[settings.toml](settings.toml)全体をコピーして、イメージ名の前に
`-v "$PWD/settings.toml:/app/settings.toml:ro"`を追加します。起動引数を変更するときは
`--bind 0.0.0.0`と`--port 29483`の両方を指定してください。上流の配布イメージはLinux amd64向けです。
他のCPUやSafeRustXNGのHTML取得なし構成では、ソースからローカルビルドしてください。

### 4. ソース

SafeRustXNGの`main`ブランチをソースから直接ビルドして起動します。

```bash
git clone --branch main --single-branch https://github.com/yuzolabs/SafeRustXNG.git
cd SafeRustXNG
cargo build --release --locked --bin saferustxng
./target/release/saferustxng --bind 127.0.0.1 --port 29483
```

生成されるバイナリは`target/release/saferustxng`（バージョン出力`SafeRustXNG 0.26.0`）です。Rust 1.88以降（推奨は現在のstable）を利用してください。

## 設定と運用

> **破壊的な名称変更と移行：** 表示名および`--version`・`/config`のバージョン接頭辞を`RustXNG`から`SafeRustXNG`へ、crate名・サーバーバイナリ名を`rustxng`から`saferustxng`へ、読み取り専用インデックスバイナリ名を`rustxng-index`から`saferustxng-index`へ、アイドル終了の環境変数を`RUSTXNG_IDLE_TIMEOUT`から`SAFERUSTXNG_IDLE_TIMEOUT`へ、リリース配布アセット名を`rustxng-<tag>-<target>`から`saferustxng-<tag>-<target>`へ改名しました（版番号`0.26.0`、既定ポート`29483`、既存の`SEARXNG_*`互換設定は不変です）。既存のスクリプト、サービス定義、コンテナタグ、エージェント自動化は新しい名称へ更新してください。

このチェックアウトの既定値は`127.0.0.1:29483`です。上流`sahenjp/RustXNG`の配布済みv0.26.0バイナリ（上流のみ・履歴情報）の既定値は8888のままですが、
上記の`--port 29483`を明示したコマンドは両方で使えます。
rdshを含む移行方法は[ポート移行ガイド](docs/PORT-MIGRATION.md)を参照してください。

既定設定はバイナリに組み込まれています。設定ファイルを指定する場合は次のように起動します。

```bash
saferustxng --config /path/to/settings.toml
```

設定項目は[settings.toml](settings.toml)と[設定の詳細](docs/PORTING.md)を参照してください。
部分設定のマージには、未指定の値が変わる既知の問題があります。下記の監査報告に記載しています。

- `/healthz`：プロセスのヘルスチェック。成功しても外部エンジンの検索成功までは保証しません。
- `/config`：登録エンジンと公開可能な設定情報。
- `/stats`、`/metrics`：実行時の統計・メトリクス。
- エンジンごとのタイムアウト、サスペンド、空結果が続いた場合の自動休止。
- `SAFERUSTXNG_IDLE_TIMEOUT=300`：5分間リクエストがなければ終了。
- 任意のValkey・Redisキャッシュと、データベースを検索するエンジン。
- 実験機能のローカルインデックス`saferustxng-index`。既定では無効で、既存インデックスの`search`と`stats`のみに対応します（HTMLクローラーは提供しません）。

[2026-11-02の監査](docs/AUDIT-2026-11-02.md)には、SSRF判定、危険な結果URL、応答サイズ制限、
キャッシュ分離などの未解決事項があります。ローカルのエージェント向け導入ではloopbackで起動し、
外部公開の前に指摘内容を確認してください。

## 検索性能の実測

> **履歴スナップショット：** 以下のSealQA測定および[BENCHMARK.md](docs/BENCHMARK.md)の数値は、汎用XPathを含む計78個のHTML取得エンジンおよび結果ページHTML取得を削除する前の記録です。エンジン削除前後での同時期・同一環境の比較や商用プロバイダーとの再比較は行っておらず、これらの履歴数値を現在の性能保証として扱うことはできません。

2026-11-02（コミット`fef998b`時点）に、当時の既定設定・上位10件・LLMなし・ローカルインデックスなしでSealQAを測定しました。

| データセット | 問数 | 参照ページ@1 | @5 | @10 | 応答中央値 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Seal-0 | 111 | 47.75% | 54.95% | 65.77% | 1.85秒 |
| Seal-Hard | 254 | 39.37% | 49.21% | 61.42% | 1.53秒 |

数値は参照ページへの到達率で、**回答の正解率ではありません**。
Seal-0はSeal-Hardの部分集合で、重複する質問には同一の観測を使用しています。
ネットワークや検索先の状態によって結果は変わります。[測定条件と全問結果](docs/SEALQA.md)を参照してください。
以前の測定や商用プロバイダーとの比較は、各データセット・条件とともに[BENCHMARK.md](docs/BENCHMARK.md)へ掲載しています。

## プロバイダー連携

エンジンのインターフェースは、ヘッダー・フォーム・JSONによる認証付きHTTP APIに対応しています。
任意のプロバイダーには、それぞれの認証情報や利用料金が必要です。
サーバー側の返却項目指定、関連段落の抽出、厳密なトークン予算は今後の課題です。
現在は`max_results`、ドメイン指定、取得後の項目抽出を利用できます。

## ドキュメント

| ドキュメント | 内容 |
| --- | --- |
| [エージェント向け導入](docs/AGENT-SETUP.md) | 導入、既存サービスの再利用、検索確認、接続先の報告 |
| [AI基盤としての設計](docs/AI-INFRA.md) | 設計意図。現在の制約は監査報告も参照 |
| [SealQA](docs/SEALQA.md) | 検索ベンチと再現可能な結果 |
| [ベンチマーク](docs/BENCHMARK.md) | 以前の性能・品質測定 |
| [互換性](docs/COMPATIBILITY.md) | SearXNGとの互換性・差異 |
| [対応範囲](docs/COVERAGE.md) | エンジンの対応範囲 |
| [ローカルインデックス](docs/LOCAL-INDEX.md) | 任意の既存BM25インデックス（`search` / `stats`のみ、HTMLクロールなし） |
| [rdsh](docs/RDSH.md) | rdshとの連携 |
| [監査](docs/AUDIT-2026-11-02.md) | 指摘事項・再現ログ・優先順位 |
| [セキュリティ](SECURITY.md) | 脆弱性の報告方法と依存関係の注意事項 |

## コミュニティとメンテナンス

質問・アイデアは[Discussions](https://github.com/yuzolabs/SafeRustXNG/discussions)へ。
招待なしでfork・PRから参加できます。継続して担当したい方は[メンテナー参加フォーム](https://github.com/yuzolabs/SafeRustXNG/issues/new?template=maintainer_application.yml)で相談してください。
[運営方針](GOVERNANCE.md)・[担当者一覧](MAINTAINERS.md)・[FAQ](docs/FAQ.md)・[ロードマップ](docs/ROADMAP.md)も参照してください。

## 開発への参加

[CONTRIBUTING.md](CONTRIBUTING.md)を参照してください。変更を次のコマンドで検証します。

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

## ライセンス

[AGPL-3.0-or-later](LICENSE)。SafeRustXNGは[SearXNG](https://github.com/searxng/searxng)を基にした[sahenjp/RustXNG](https://github.com/sahenjp/RustXNG)から派生しています。
帰属情報は[NOTICE](NOTICE)に記載しています。改変版のサービス運用時のソース提供等についてはライセンス本文を参照してください。
