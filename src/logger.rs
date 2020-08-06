use std::time::Instant;
use fern::colors::{ColoredLevelConfig, Color};
use log::{LevelFilter, Level};

/// Start the logger, hiding every message whose level is under the provided one
pub fn start(level: LevelFilter) {
    // Create color scheme
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Cyan)
        .trace(Color::Blue);

    // Get instant
    let started = Instant::now();

    // Build the logger
    fern::Dispatch::new()
        .format(move |out, message, record| {
            let elapsed = started.elapsed();
            let secs = elapsed.as_secs();

            out.finish(format_args!(
                "{}[{: >2}m {: >2}.{:03}s] {}: {}\x1B[0m",
                format_args!("\x1B[{}m", colors_line.get_color(&record.level()).to_fg_str()),
                secs / 60,
                secs % 60,
                elapsed.subsec_millis(),
                match record.level() {
                    Level::Info => "INFO",
                    Level::Warn => "WARNING",
                    Level::Error => "ERROR",
                    Level::Debug => "VERBOSE",
                    Level::Trace => "DEBUG"
                },
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()
        .unwrap()
}
