use tracing::{metadata::LevelFilter, Level, Metadata};
use tracing_subscriber::layer::{Context, Filter};

pub struct LogFilter {
    level: Level,
}

impl LogFilter {
    pub fn new(level: Level) -> Self {
        LogFilter { level }
    }
}

impl<S> Filter<S> for LogFilter {
    fn enabled(&self, meta: &Metadata<'_>, _cx: &Context<'_, S>) -> bool {
        let target = meta.target();

        // enable logging for the app itself, and for tower_http response log
        target.starts_with("data_exporter") || target.starts_with("tower_http::trace::on_response")
    }

    fn max_level_hint(&self) -> Option<LevelFilter> {
        Some(LevelFilter::from(self.level))
    }
}
