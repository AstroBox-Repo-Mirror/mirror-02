//! tracing 日志初始化(输出到宿主 stdout,无需文件日志)。

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() {
    let console_layer = fmt::layer()
        .with_target(true)
        .with_ansi(true)
        .with_writer(|| std::io::stdout())
        .compact();
    tracing_subscriber::registry().with(console_layer).init();
}
