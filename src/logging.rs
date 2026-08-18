//! ログ出力の初期化。標準出力と `logs/` 配下のファイルの両方に書く。

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// `RUST_LOG` が未設定のときのログレベル。
const DEFAULT_FILTER: &str = "todo_app=debug,tower_http=info,sqlx=warn";

/// ログ出力先のディレクトリとファイル名の接頭辞。
/// 実際のファイルは `logs/todo-app.log.YYYY-MM-DD` になる。
const LOG_DIR: &str = "logs";
const LOG_PREFIX: &str = "todo-app.log";

/// 戻り値の `WorkerGuard` は main が終わるまで保持すること。
/// drop すると書き込みスレッドが止まり、未書き出しのログが失われる。
pub fn init() -> WorkerGuard {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));

    let file_appender = tracing_appender::rolling::daily(LOG_DIR, LOG_PREFIX);
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(filter)
        // 標準出力（色つき）
        .with(tracing_subscriber::fmt::layer())
        // ファイル（色コードは混ぜない）
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file_writer)
                .with_ansi(false),
        )
        .init();

    guard
}
