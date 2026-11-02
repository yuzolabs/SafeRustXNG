# rdsh から使う

rdsh は `search-web` のバックエンドとして SafeRustXNG（SearXNG 互換 JSON API）を使えます。
このチェックアウトのバイナリ名は `saferustxng`、既定ポートは 29483 です。旧来の 8888 探索や `rustxng` コマンド探索に依存せず、`saferustxng` を起動したうえで `RUSTXNG_URL` を明示してください。

## 使い方

```bash
# 1. SafeRustXNG をポート 29483 で起動
saferustxng --bind 127.0.0.1 --port 29483

# 2. rdsh から検索（外部 rdsh の --engine rustxng / RUSTXNG_URL はそのまま使用）
RUSTXNG_URL=http://127.0.0.1:29483 rdsh search-web --engine rustxng "python dict get"
RUSTXNG_URL=http://127.0.0.1:29483 rdsh search-web --engine rustxng --json "python dict get"
```

## 解決順

`rdsh search-web --engine auto`（既定）は次の順に探します。

1. `SEARXNG_URL`
2. `RUSTXNG_URL`
3. ローカルの `rustxng` コマンド
4. `http://127.0.0.1:8888`

SafeRustXNG のバイナリ名は `saferustxng`、既定ポートは `29483` のため、3 と 4 の既定探索には一致しません。SafeRustXNG を動かす場合（別ホスト・別ポートを含む）は `RUSTXNG_URL` を明示してください:

```bash
RUSTXNG_URL=http://127.0.0.1:9999 rdsh search-web --engine rustxng "query"
```

## バイナリの導入

外部の `rdsh rustxng install` は上流 `sahenjp/RustXNG` のリリースアセット名（`rustxng-<tag>-<target>`）を前提としているため、このフォークの改名済みアセット（`saferustxng-<tag>-<target>`）や SafeRustXNG の `main` ブランチをインストールすることはできません。SafeRustXNG を使用する場合は、[README](../README.ja.md#導入)の手順でソースから `saferustxng` をビルドして起動し、`RUSTXNG_URL=http://127.0.0.1:29483` を明示して接続してください。

## 互換性

rdsh は SearXNG の JSON API を読むため、SafeRustXNG は同じ形で応答します。

- `/search?q=...&format=json` の `results[]`（`title` / `url` / `content`）
- `/healthz`（起動確認）
- `unresponsive_engines` は `[engine, 理由]` の配列

2026-10-31 時点で、`rdsh search-web --engine rustxng --json` が当時の `rustxng` 応答をそのまま解釈できることを実機で確認しています。

## rdsh と併用するときの推奨設定

- **自前インデックスは任意** — `search.local_index_path`を設定した場合だけ利用します。ディレクトリを置くだけでは有効になりません。
- **キャッシュ** — rdsh は同じクエリを繰り返しがちです。既定のキャッシュ（300 秒）で 2 回目以降は 16 ms で返ります
- **`--limit`** — rdsh 側の件数指定に合わせて `max_results` が渡ります

## サイドカーの後始末

一時的なバックグラウンドプロセスが残り続けるのを防ぐため、SafeRustXNG には**アイドル終了**を用意しています。

```bash
# 5 分間リクエストが無ければ自動で終了する
SAFERUSTXNG_IDLE_TIMEOUT=300 saferustxng
```

`settings.toml` の `server.idle_timeout` でも指定できます（既定 0 = 無効。旧環境変数 `RUSTXNG_IDLE_TIMEOUT` は `SAFERUSTXNG_IDLE_TIMEOUT` に改名されました）。
