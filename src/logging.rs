//! ログ出力の初期化。標準出力と設定されたディレクトリのファイルの両方に書く。

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;

/// ログファイル名の接頭辞。実ファイルは `<log_dir>/todo-app.log.YYYY-MM-DD` になる。
const FILE_PREFIX: &str = "todo-app.log";

/// 戻り値の `WorkerGuard` は main が終わるまで保持すること。
/// drop すると書き込みスレッドが止まり、未書き出しのログが失われる。
pub fn init(config: &Config) -> WorkerGuard {
    let file_appender = tracing_appender::rolling::daily(&config.log_dir, FILE_PREFIX);
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::new(&config.log_filter))
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
