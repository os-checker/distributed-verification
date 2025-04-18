use tracing_subscriber::{EnvFilter, fmt, prelude::*, registry};

pub fn init() {
    // let fmt_layer = fmt::layer();
    let fmt_layer = fmt::layer()
        .with_line_number(false)
        .with_level(false)
        .without_time()
        .with_file(false)
        .with_target(false);
    let env_layer = EnvFilter::from_default_env();
    let error_layer = tracing_error::ErrorLayer::default();

    if let Err(err) = registry().with(env_layer).with(error_layer).with(fmt_layer).try_init() {
        eprintln!("Logger already init: {err}");
    };

    color_eyre::install().unwrap();
}
