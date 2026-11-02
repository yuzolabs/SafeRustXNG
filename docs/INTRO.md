# RustXNG — AI エージェントに使わせる統合検索エンジン

> 検索 API を 1 本に。複数の検索先を、自分のサーバーから。
>
> **現在の構成に関する注意：** 人向け HTML ページを取得・解析する計 78 エンジン（汎用 `xpath` エンジン、および Google・Bing・DuckDuckGo・Brave の HTML 検索などを含む）と、検索結果ページの HTML 本文取得（`include_raw_content` / `raw_content`）・HTML クローラー（`rustxng-index crawl`）は、SafeRustXNG（<https://github.com/yuzolabs/SafeRustXNG/tree/main>）で削除されています。現在は非 HTML レスポンス（JSON API・Atom/RSS フィード・DB 等）を扱うエンジンのみを残しています（公式 API 限定や利用規約への適合を保証するものではありません）。下表のレイテンシ等の数値は HTML エンジン削除前の履歴スナップショットです。エンジン削除前後での同時期・同一環境の比較や商用プロバイダーとの再比較は行っておらず、履歴数値を現在の性能保証として扱うことはできません。

## これは何か

**RustXNG は、AI エージェントに検索させるためのセルフホスト型メタ検索エンジンです。**

Wikipedia・GitHub・arXiv・PubMed・crates.io・Google News（RSS）・Bing News（RSS）など、非 HTML の API・フィードを検索するエンジン群（`settings.toml` に 183 エントリ、既定有効 98）を 1 つの JSON API にまとめ、Rust の単一バイナリで動きます。Python もデータベースも必須の API キーも要りません。

SearXNG の基本的な HTTP API・検索構文・設定形式を引き継いでいるため、既存の SearXNG 向けツールやプロンプトから利用できます（削除された HTML エンジンや `include_raw_content` 等の例外については [docs/COMPATIBILITY.md](COMPATIBILITY.md) を参照してください）。

## なぜ AI エージェントに良いのか

### 1. ツール呼び出しに向いた軽量構成（履歴測定値）

| 指標（HTML エンジン削除前の履歴スナップショット） | 数値 |
| :--- | ---: |
| 起動（履歴） | 0.03 秒 |
| 常駐メモリ（履歴） | 13 MiB |
| 単一サイト検索（ウォーム・履歴） | **1 ms** |
| 全サイト検索（HTML エンジン削除前のウォーム中央値・履歴） | **276 ms** |

エージェントは 1 ターンで何度も検索するため、検索サーバー自体のオーバーヘッドやキャッシュ応答の速さが重要になります。上表は HTML エンジン削除前に計測された履歴値であり、現在の SafeRustXNG 構成での測定値ではありません（詳細は [docs/BENCHMARK.md](BENCHMARK.md) を参照してください）。

### 2. 検索先をまたぐ共通の JSON 結果スキーマ

レスポンスは各エンジンの結果を共通の JSON 形式（`query` / `results` / `answers` / `infoboxes` / `suggestions` / `unresponsive_engines`）に統合して返します。検索先ごとに個別のレスポンス形式を扱う必要がありません。

```json
{
  "query": "rust async runtime",
  "results": [
    {
      "url": "https://tokio.rs/",
      "title": "Tokio — an asynchronous Rust runtime",
      "content": "Tokio is an event-driven, non-blocking I/O platform...",
      "engine": "github",
      "engines": ["github", "mwmbl"],
      "score": 1.0,
      "category": "general"
    }
  ],
  "unresponsive_engines": [["wikidata", "Timeout"]]
}
```

計測値が必要なときは `&full=1` を付けると `timings` / `number_of_results` / `engines` が増えます。障害時の切り分けに使えます。

### 3. サイト指定・新着指定が URL 1 本で済む

```bash
# GitHub だけを検索（!サイト名）
curl 'http://127.0.0.1:29483/search?q=%21github+tokio&format=json'

# 24 時間以内の情報だけ（ニュース系エンジンが日付フィルタを解釈）
curl 'http://127.0.0.1:29483/search?q=ai&time_range=day&format=json'

# カテゴリ指定（general / news / images / videos / science / it / files ... 全 47 種）
curl 'http://127.0.0.1:29483/search?q=llm&categories=science&format=json'
```

「まず GitHub だけ見る」「最新ニュースだけ」といったエージェントの判断を、追加の API なしで表現できます。

### 4. 落ちているサイトを自動で切り離す

応答しないサイトは自動でサスペンドし、以降の検索から外します。1 つのサイトの障害でツール全体のレイテンシが崩れません。障害中のエンジンは `unresponsive_engines` に入るので、エージェント側で「この情報源は今回使えなかった」と説明できます。

### 5. 自前のデータも同じ API で引ける

PostgreSQL / MySQL / MariaDB / MongoDB / SQLite / Valkey / Elasticsearch / Solr / Meilisearch を検索対象にできます。社内 Wiki やドキュメントを DB に入れておけば、**Web 検索と社内検索を同じ JSON で**エージェントに渡せます。

```toml
[[engines]]
name = "社内ドキュメント"
engine = "postgresql"
shortcut = "docs"
categories = ["general", "it"]
host = "127.0.0.1"
database = "wiki"
query_str = "SELECT title, body FROM docs WHERE body LIKE '%' || $1 || '%'"
```

### 6. プライバシーとコスト

RustXNG 自体はセルフホストで動作し、独自の広告・追跡・プロファイル作成は行いません。多くのエンジンは有料 API キーなしで利用できます。
一方で、**検索クエリはリクエストで選択された外部の検索プロバイダーへ送信される**ため、RustXNG サーバーの IP アドレスや検索語は各プロバイダーに届き、プロバイダー側でのログ記録や保持期間は各事業者のポリシーに依存します。また、画像プロキシ（`image_proxy`）は任意機能であり既定では無効（`settings.toml` で `image_proxy = false`）です。

## エージェントへの組み込み方

### 関数呼び出しのツール定義

```json
{
  "name": "web_search",
  "description": "複数の検索エンジンを横断して Web を検索する。最新情報が必要なときは time_range を day にする。特定サイトだけを見たいときは query の先頭に !github のように付ける。",
  "parameters": {
    "type": "object",
    "properties": {
      "query": { "type": "string", "description": "検索語。例: 'rust tokio', '!github tokio'" },
      "time_range": { "type": "string", "enum": ["day", "week", "month", "year"] },
      "categories": { "type": "string", "description": "general / news / science / it / files など" },
      "max_results": { "type": "integer", "description": "返す最大件数。トークンを抑えたいときは 5 を指定" },
      "include_domains": { "type": "string", "description": "このドメインだけに絞る（カンマ区切り）" },
      "exclude_domains": { "type": "string", "description": "このドメインを除外する（カンマ区切り）" },
      "pageno": { "type": "integer", "minimum": 1 }
    },
    "required": ["query"]
  }
}
```

エンドポイントは `GET /search?q=...&format=json` だけです。MCP で使いたい場合は、この 1 本を叩く薄い MCP サーバーを被せれば動きます（組み込みの MCP サーバーはまだありません）。

### トークンを抑える

エージェントに渡す量は、パラメータ 1 つで半分以下にできます。

```bash
# 既定: 全結果（例: 60 KB / 200 件）
curl 'http://127.0.0.1:29483/search?q=rust&format=json'

# max_results: 上位 5 件だけ（例: 4.6 KB / -92%）
curl 'http://127.0.0.1:29483/search?q=rust&format=json&max_results=5'
```

検索結果ページの HTML 本文取得（旧 `include_raw_content` / `raw_content`）は削除されており、API・フィードが返す要約（`content`）のみを返します。

## 使い始める

HTML 取得なし構成は SafeRustXNG の `main` ブランチ（<https://github.com/yuzolabs/SafeRustXNG/tree/main>）からのソース導入として提供しています。ソースからビルドして起動してください（生成されるバイナリ名は `rustxng`、バージョン表記は `0.26.0` のままのため、バージョン表示だけでは上流ビルドと区別できません）。上流 `sahenjp/RustXNG` の `v0.26.0` リリースバイナリや公開コンテナイメージは上流のみ（履歴情報）であり、SafeRustXNG の変更が含まれることは確認されていません。

```bash
git clone --branch main --single-branch https://github.com/yuzolabs/SafeRustXNG.git
cd SafeRustXNG
cargo build --release --locked --bin rustxng
./target/release/rustxng --bind 127.0.0.1 --port 29483
```

上記チェックアウトから Docker イメージをローカルビルドする場合:

```bash
docker build -t rustxng . && docker run --rm -p 127.0.0.1:29483:29483 rustxng
```

`http://127.0.0.1:29483/` で検索画面、`/search?q=...&format=json` で API です。

## 補足

- **ライセンス**: AGPL-3.0-or-later（SearXNG と同じ）。帰属は NOTICE に記載
- **互換性**: SearXNG 互換の HTTP API・検索構文・設定形式。汎用 `xpath` を含む計 78 個の HTML 取得エンジンは削除済みで、現在は非 HTML エンジン（165 モジュール / `settings.toml` 183 エントリ）のみです。詳細は [docs/COMPATIBILITY.md](COMPATIBILITY.md) と [docs/COVERAGE.md](COVERAGE.md)
- **バイナリサイズ**: 17 MB（DB エンジンを外した場合）/ 27 MB（全部入り）
- **向いていない用途**: 画面デザインは独自なので、SearXNG のテーマをそのまま使いたい場合は不向きです
