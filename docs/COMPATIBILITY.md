# SearXNG 互換性の範囲

**結論: プロトコル（HTTP API・検索構文・設定・データ形式）は上流と一致しています。** 一方、人向け HTML ページを取得・解析する計 78 エンジン（汎用 `xpath` エンジンを含む）、および検索結果ページの HTML 本文取得（`include_raw_content` / `raw_content`）・HTML クローラー（`rustxng-index crawl`）は削除されています。残っている非 HTML（JSON API・Atom/RSS フィード・DB 等）エンジンは公式 API 限定や利用規約への適合を保証するものではなく、エンジン削除後の検索品質の比較可能性は再測定していません。残る主な差分はエンジン対象範囲、UI のマークアップ、i18n のロケール数（57 / 60）です。このページは「何が互換で、何が違うか」を検証可能な形でまとめたものです。

## 互換と言える範囲

| 領域 | 状態 | 根拠 |
| :--- | :--- | :--- |
| 検索構文（`!engine` / `!category` / `:lang` / `<timeout` / `!!` / `!!bang`） | **互換** | 上流 `searx/query.py` と同じ判定順・同じ正規化 |
| 結果の統合（重複排除・スコア・カテゴリ別グループ化） | **互換** | `searx/results.py` の `calculate_score` / `get_ordered_results` と同じ式 |
| `/search` のパラメータ名・出力形式（html/json/csv/rss） | **互換** | 統合テストで検証 |
| JSON API の形 | **互換（既定）** | 上流 `get_json_response` の 7 フィールドのみを出力。詳細版は `full=1` |
| JSON の各要素の形 | **互換** | `answers` = `[{"answer":...}]`、`unresponsive_engines` = `[[engine, "Translated error"]]`、`template` = `"default.html"`、`metadata` = 文字列、`priority` 既定は空文字 |
| CSV 出力 | **互換** | 上流 `write_csv_response` と同じ列（`title,url,content,host,engine,score,type`）と同じ行種別（result / answer / suggestion / correction） |
| `/config` | **互換** | 上流と同じフィールド（`brand`、`limiter`、`locales`、`engines[].timeout` などを含む） |
| 設定ファイル | **互換（YAML をそのまま読める）** | `--config settings.yml` / `SEARXNG_SETTINGS_PATH` に上流の YAML をそのまま指定可能。TOML も可 |
| 環境変数（`SEARXNG_*`） | **互換** | 同じ名前を受け付け |
| 外部 bang | **データ互換** | 上流の 13,561 件を同一データから生成 |
| Cookie 名 | **互換** | `category` / `language` / `locale` / `safesearch` / `theme` / `disabled_engines` など上流と同じ（旧名 `categories` も読み取り可能） |
| OpenMetrics のエンジン指標名 | **互換** | `searxng_engines_*` |

## JSON API

既定の `format=json` は上流の形だけを返します。

`length` / `iframe_src` / `audio_src` / `views` など本実装が持たないフィールドは、エンジン側の `metadata` 文字列に含まれます（上流と同じ「メタデータ」表現）。

計測値を含む詳細版が必要な場合は `format=json&full=1` を使います（`number_of_results` / `timings` / `engines` / `engine_data` / `paging` / `redirect_url` / `error` が追加されます）。

## エージェント向けの拡張（上流にないパラメータ）

SearXNG のパラメータと衝突しない名前で、LLM のツール呼び出し向けの指定を追加しています。省略すれば従来の挙動のままです。

| パラメータ | 意味 | 返り値への影響 |
| :--- | :--- | :--- |
| `max_results=N` | 並べ替え後の上位 N 件だけを返す（1〜200 に丸める） | `results` を切り詰め |
| `include_domains=a.com,b.com` | URL のホストが指定ドメイン（サブドメイン含む）の結果だけを残す | 絞り込み |
| `exclude_domains=a.com` | 指定ドメインの結果を落とす | 絞り込み |
| `strategy=staged` | エンジンを少数の段に分けて開始し、`search.staged_min_results` 件で打ち切る（既定は並列） | 応答時間は短縮、返る件数は減る |

検索結果ページの HTML 本文取得（旧 `include_raw_content` パラメータおよび `SearchResult.raw_content` フィールド、`search.raw_content_limit` / `search.raw_content_chars` 設定）は削除されました。旧クエリパラメータが指定された場合も無視され、結果 URL への通信は行いません。

**ランキングの差分（`search.relevance_weight`）**: 一般（マルチエンジン）検索では、URL + タイトル + スニペット（`content`）のクエリ語一致度でスコアを補正します（既定 4.0）。**単一エンジン指定では補正せず上流順を維持**するので、`engines=` での overlap@10 は上流と一致します。`relevance_weight = 0` にすれば一般検索も上流と完全に同一の順序になります。

応答自体の検証も行います。パースは通るが内容が明らかに架空・プレースホルダ・クエリと無関係な応答（上流 issue #6358 の「静かなゴミ混入」）は、結果に混ぜずにエンジンをサスペンドし、`unresponsive_engines` に `Suspicious response` として出します。

### ディープリサーチ（`GET /research`、上流にない）

質問を受け取り、(1) サブクエリを計画（LLM があれば LLM、無ければ質問から派生）、(2) 各サブクエリを検索してエンジン（API・フィード）が返した `content`（要約）を収集・URL 重複排除（結果先 URL の HTML ページ取得は一切行いません）、(3) LLM があれば引用付き markdown レポートを生成して返します。LLM 未設定でも証拠バンドル（`queries` / `sources`）は返ります。設定は `settings.toml` の `[research]`（`enabled` / `base_url` / `api_key` / `model` / `max_queries` / `max_sources`）。

## 互換でない領域

| 領域 | 差分 |
| :--- | :--- |
| **エンジンの網羅率** | 当初は上流 352 エントリ / 226 型を移植しましたが、その後汎用 `xpath` を含む計 78 個の HTML 取得エンジンを削除しました。現在は非 HTML レスポンスを扱う 165 モジュール / `settings.toml` 183 エントリ（既定有効 98）です。内訳は [COVERAGE.md](COVERAGE.md) |
| **UI** | 単一テーマ（simple 相当）のみ。上流のテンプレート・テーマ資産とマークアップは非互換 |
| **i18n** | 57 ロケール（上流は 60） |
| **`/stats`** | 表示形式が上流と異なる（HTML の内容・`/stats/errors` の形） |
| **サブシステム** | プラグイン 11 / 上流 12、favicon は上流の resolver 方式ではない |
| **キャッシュ** | メモリ内（TTL + LRU）と Valkey / Redis の両方に対応 |
| **botdetection** | 信頼プロキシの XFF 解決、IP 制限のスライディング窓、リンクトークン、429 応答 |
| **DB 直結エンジン** | 11 種すべて実装済み（db-engines フィーチャー、既定で有効） |

## 検証の方法

- 構文・統合ロジック・JSON/CSV の形: `cargo test`
- 上流との過去の実測比較（HTML エンジン削除前の履歴スナップショット）: [BENCHMARK.md](BENCHMARK.md) と `bench/compare.py`（エンジン削除後の検索品質・比較可能性は未再測定）
- エンジン網羅率: `python3 tools/coverage.py /path/to/searxng settings.toml`

## 完全互換に近づけるには

1. エンジン型は当初 226/226 を移植した後、汎用 `xpath` を含む計 78 個の HTML 取得エンジンを削除済み（DB ドライバ直結の 8 種は `db-engines` フィーチャーで実装）
2. `/stats` と UI テンプレートを上流資産に合わせる
3. i18n のテンプレート全面適用とロケール拡大
4. UI テンプレートのマークアップを上流に合わせる

## 商用 API をエンジンとして併用する

RustXNG はメタ検索のハブなので、商用検索 API を登録エンジンの 1 つとして足せます。API のカバレッジと RustXNG 側のランキング（関連度・権威・スニペット）を組み合わせられます。

`exaapi` / `braveapi` は実装済みで既定は無効です。設定ファイルで有効化します。

```toml
# my-settings.toml
use_default_settings = true

[[engines]]
name = "exaapi"
engine = "exaapi"
shortcut = "exa"
categories = ["general", "web"]
disabled = false          # 既定で無効なエンジンを有効化できる
api_key = "YOUR_EXA_KEY"
content_mode = "highlights"   # highlights | text
```

```bash
SEARXNG_SETTINGS_PATH=my-settings.toml ./rustxng --port 29483
```

> 0.13.0 で `EngineConfig::disabled` を三状態（`Option<bool>`）にしました。以前は `disabled = false` が「未指定」と区別できず、既定で無効なエンジンを有効化できませんでした。

### 実測（100 問・履歴スナップショット）

> **履歴スナップショット：** 下表は汎用 `xpath` を含む計 78 個の HTML 取得エンジンを削除する前の測定記録であり、エンジン削除後の検索品質や比較可能性は再測定していません。

| 構成 | hit@1 | hit@5 | MRR |
| --- | ---: | ---: | ---: |
| Exa 単体（API 直） | 74 | **88** | **0.808** |
| **RustXNG + exaapi** | **75** | 85 | 0.799 |
| RustXNG 単体（4 回平均） | 62 | 79 | 0.710 |

API を併用すると **hit@1 は Exa 単体を上回ります**（75 対 74）。hit@5 と MRR は僅差で Exa が上（−3 問 / −0.009）で、全体としてはほぼ同等です。上流エンジンがブロックされる環境では、API の安定したカバレッジが効きます。
