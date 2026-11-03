# エンジン移植の状況（カバレッジ）

現在は、人向け HTML ページを取得・解析する計 78 エンジン（汎用 `xpath` エンジン［XML 用を含む］、および HTML 内の埋め込み JSON を抽出する実装を含む）を削除しています。あわせて、検索結果ページの HTML 本文取得（`include_raw_content` / `raw_content`、`src/content.rs`）、エンジン向け HTML/XPath パーサ（`src/engine/xpath.rs` / `src/engine/xml.rs`、`scraper` 依存）、および `rustxng-index crawl` の HTML クローラーも削除しました（現在の `saferustxng-index`［旧 `rustxng-index`］は既存ローカルインデックスの `search` と `stats` のみに対応し、Web の HTML UI 出力と API・フィード要約のタグ/エンティティ除去は引き続き提供します）。arXiv・RSS・API など非 HTML レスポンスを扱うエンジンは残していますが、公式 API のみの構成や利用規約への適合を保証するものではありません。また、エンジン削除前後での同時期・同一環境の比較や商用プロバイダーとの再比較は行っておらず、履歴数値を現在の性能保証として扱うことはできません。

さらに、自動取得・非公開 API の利用・検索結果の再表示・個人用 reader 以外の RSS 利用が規約で禁止される連携を除去するため、非 HTML の 30 型（既定有効 22・無効 8）を削除しました。Piped / Invidious も、[YouTube API Services Developer Policies](https://developers.google.com/youtube/terms/developer-policies) のスクレイプデータ取得禁止との整合のため削除しています。型・shortcut の一覧と移行時の動作は [CHANGELOG](../CHANGELOG.md#unreleased) を参照してください。Google 検索候補バックエンドも削除しましたが、エンジンの新規追加はありません。残る連携がすべて公式 API であることや規約に適合することを保証するものではありません。

現在は 135 エンジンモジュール / `settings.toml` 153 エントリ（既定有効 76）です。現在の登録は `src/engines/mod.rs`、同梱設定は `settings.toml` を参照してください。以下の数値は削除前の記録であり、現在の対応数やテスト数ではありません。上流との比較は `tools/coverage.py` で再計測できます。

```bash
python3 tools/coverage.py /path/to/searxng settings.toml
```

## 削除前の数値（履歴）

| 指標 | 値 |
| --- | ---: |
| 上流のエンジン型 | 226 |
| 本移植のエンジン型 | **226 / 226（100%）** |
| 上流エントリのうち型を実装済み | **352 / 352（100%）** |
| 上流にない追加エンジン | yahoo_japan（日本語検索） |
| 本移植の設定エントリ | 275 |
| 既定で有効なエントリ | 約 170（認証情報・自前インスタンスが必要なものは上流と同じく無効） |
| テスト | ユニット 722 + 統合 14 |

## 移植を支える仕組み

1. **宣言的エンジン**（`engine: json_engine` / `mediawiki`）で上流 `settings.yml` の定義をそのまま取り込めます。
2. **コードエンジン**は 1 エンジン 1 ファイル（`src/engines/<name>.rs`）。すべてに外部ネットワーク不要のフィクスチャテストを付けています。
3. **外部 bang の全量移植**: 上流の圧縮トライ（13,561 件）を平坦化。
4. **登録の自動化**: `tools/register_engines.py` が `// engine-id:` 行を読み、`mod.rs` の宣言・ディスパッチと `settings.toml` のエントリを同期します。
5. **壊れたエンジンは起動時にスキップ**: 上流と同じく、初期化に失敗したエンジンは警告を出して除外し、残りで検索を続けます。

## 削除前のエンジン型の内訳（履歴）

次の表は過去の分類・状態を保存したもので、現在利用できるエンジンの一覧ではありません。

| 分類 | 数 | 状態 |
| --- | ---: | --- |
| スクレイパー / HTTP API | 約 200 | 既定で有効、または認証情報を設定すれば有効 |
| 検索バックエンド（HTTP） | 3 | `elasticsearch` / `sorl` / `meilisearch`（`base_url` + index / collection を設定） |
| DB ドライバ直結 | 8 | `postgresql` / `mysql_server` / `mariadb_server` / `mongodb` / `sqlite` / `valkey_server` / `recoll` / `opensemantic` — 設定を読んで説明用のエラーを返すスタブ |
| MediaWiki / 宣言型 | 40 以上 | `tools/import_engines.py` で上流定義から生成 |

### DB ドライバ直結エンジンについて

`postgresql` / `mysql_server` / `mariadb_server` / `mongodb` / `sqlite` / `valkey_server` / `recoll` / `opensemantic` は、SearXNG が DB ドライバを直接使ってインデックスを検索するエンジンです。Rust 版では DB クライアントの依存を増やさない方針のため、設定（接続先・認証情報）を読み取り、実際の検索時には理由付きのエラーを返すスタブとして実装しています。HTTP 経由で検索できる `elasticsearch` / `sorl` / `meilisearch` は完全実装です。

### 実装済みでも既定で無効なもの

上流と同じく、次は認証情報や自前インスタンスの設定が必要なため `disabled = true` にしています。

- API キー: `braveapi`, `kagi`, `exaapi`, `wolframalpha_api`, `springer`, `flickr`, `youtube_api`, `giphy`, `marginalia`, `jina`, `spotify` など
- 自前インスタンス: `lemmy`, `mastodon`, `peertube`, `sepiasearch`, `discourse`, `torznab`, `yacy`
- 翻訳系: `deepl`, `libretranslate`, `lingva`, `mozhi`, `translated`

## 移植時の意図的な差分

- **2 リクエスト必要なエンジン**（PubMed の esearch→efetch、Open-Meteo のジオコーディングなど）は、単一リクエストで取得できる代替エンドポイントを使うか、トークン等を設定値から受け取ります。各ファイル冒頭に明記しています。
- **JavaScript 実行が必要なページ**は対象外、または JSON エンドポイントに切り替えています。
- **アシスタント型エンジン**（`calculator` など）はプラグイン / アンサーラーとして実装しています。
- **天気・通貨・翻訳**は answerer / engine のどちらでも扱えるよう、アンサーラーと engine の両方を用意しています。

## サブシステム（エンジン以外）

| 上流 | 状態 |
| --- | --- |
| プラグイン 12 種 | 11 移植済み（hostnames / tracker_url_remover / time_zone / calculator / hash / oa_doi / self_info / unit_converter / ahmia_filter / tor_check / infinite_scroll） |
| アンサーラー | statistics / random / weather |
| favicons | 実装（`/favicon_proxy`、キャッシュ付き） |
| i18n | 57 ロケール（`tools/import_translations.py` で生成） |
| wikidata インフォボックス | `src/wikidata_infobox.rs` |
| valkey / redis キャッシュ | 未移植（TTL + LRU のメモリ内キャッシュで代替） |
| botdetection / limiter.toml | 部分（クライアント単位の簡易レート制限のみ） |
