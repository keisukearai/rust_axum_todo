# rust_axum_todo

Rust + [Axum](https://github.com/tokio-rs/axum) で書いた TODO アプリ。JSON API と素の HTML/JS 画面がセットになっています。
永続化は PostgreSQL（[sqlx](https://github.com/launchbadge/sqlx)）。

## 必要なもの

- Rust（stable。[rustup](https://rustup.rs/) で導入）
- PostgreSQL 16 以上

## セットアップ

```sh
# 1. データベースを作る
createdb rust_axum_todo

# 2. 接続情報を書く
cp .env.example .env
$EDITOR .env        # DATABASE_URL を自分の環境に合わせる

# 3. 起動（テーブルは起動時に自動で作られる）
cargo run
```

http://127.0.0.1:3000 が開きます。

マイグレーションは `migrations/` を起動時に自動適用します。適用済みかどうかは `_sqlx_migrations` テーブルで判定されるので、二重に実行されることはありません。

## 設定

すべて環境変数で、`.env` に書きます（`.env.example` を参照）。読み取りと検証は `src/config.rs` にまとまっています。

| 変数 | 既定値 | 内容 |
|---|---|---|
| `DATABASE_URL` | **必須** | PostgreSQL の接続先 |
| `HOST` | `127.0.0.1` | 待ち受けアドレス。コンテナで動かすなら `0.0.0.0` |
| `PORT` | `3000` | 待ち受けポート |
| `DATABASE_MAX_CONNECTIONS` | `5` | コネクションプールの上限 |
| `LOG_DIR` | `logs` | ログファイルの出力先 |
| `RUST_LOG` | `todo_app=debug,tower_http=info,sqlx=warn` | ログレベル |

解釈できない値は起動時に弾き、どのキーが悪いかを示して終了します。

```
$ PORT=http cargo run
configuration error: PORT is invalid: "http" (invalid digit found in string)
```

## API

| メソッド | パス | 内容 |
|---|---|---|
| `GET` | `/api/todos` | 一覧を返す |
| `POST` | `/api/todos` | 追加する（`201`） |
| `POST` | `/api/todos/{id}` | 完了状態を反転する |
| `DELETE` | `/api/todos/{id}` | 削除する（`204`） |

リクエストとレスポンス:

```sh
$ curl -X POST localhost:3000/api/todos \
    -H 'Content-Type: application/json' \
    -d '{"title":"Rustを学ぶ"}'
{"id":1,"title":"Rustを学ぶ","done":false}

$ curl -X POST localhost:3000/api/todos/1
{"id":1,"title":"Rustを学ぶ","done":true}
```

エラーは JSON で理由を返します。

```sh
$ curl localhost:3000/api/todos/999 -X DELETE
{"error":"todo 999 not found"}
```

`title` は前後の空白を落としたうえで、空文字と 200 文字超を `400` で弾きます。

## テスト

```sh
cargo test
```

`#[sqlx::test]` がテストごとに使い捨てのデータベースを作り、`migrations/` を適用し、終了後に削除します。
テスト間で状態が混ざらないので、並列に走っても順序に依存しません。

`DATABASE_URL` が指すサーバーに `CREATE DATABASE` 権限が必要です。

## ログ

標準出力と `<LOG_DIR>/todo-app.log.YYYY-MM-DD` の両方に出ます（日次ローテーション、ファイル側は色なし）。

```sh
RUST_LOG=debug cargo run                 # レベルを変える
LOG_DIR=/var/log/todo-app cargo run      # 出力先を変える
```

既定の `logs/` はリポジトリ直下なので `.gitignore` してあります。常駐させる場合は `LOG_DIR` を
リポジトリの外に向けてください。

なお日付の切り替わりは UTC 基準です。ファイル名の日付がローカル日付と1日ずれることがあります。

## 構成

```
src/
├── main.rs        起動・DB 接続・マイグレーション適用
├── lib.rs         app() でルーターを組み立てる（テストもここを使う）
├── config.rs      環境変数の読み取りと検証
├── error.rs       AppError → ステータスコードと JSON への変換
├── logging.rs     ログ初期化
├── state.rs       AppState（コネクションプール）
└── todo/
    ├── mod.rs     /api/todos のルート定義
    ├── model.rs   Todo, CreateTodo と入力検証
    ├── repo.rs    SQL（DB アクセスはここだけ）
    └── handler.rs HTTP の入口
tests/api.rs       統合テスト
migrations/        スキーマ定義
static/            フロントエンド
```

SQL を `repo.rs` に閉じ込めてあるので、ハンドラは SQL を知りません。

## 開発メモ

SQL を書き換えたら、オフライン用のクエリ情報を作り直してコミットしてください。

```sh
cargo sqlx prepare
```

`query_as!` はコンパイル時に実データベースへ照合して SQL とカラム型を検証します。
`.sqlx/` にその結果を置いてあるので、データベースが無い環境でもビルドが通ります。
再生成を忘れると、そうした環境でビルドが落ちます（`cargo install sqlx-cli` が必要）。
