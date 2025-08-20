use std::{env, fs};
use tracing_subscriber::{EnvFilter, fmt, prelude::*, registry};

const ENV_LOG: &str = "DV_LOG";
const ENV_LOG_FILE: &str = "DV_LOG_FILE";

pub fn init() {
    let fmt_layer = fmt::layer()
        .with_line_number(false)
        .with_level(false)
        .without_time()
        .with_file(false)
        .with_target(false);

    let env_layer = EnvFilter::from_env(ENV_LOG);
    let error_layer = tracing_error::ErrorLayer::default();

    let reg = registry().with(env_layer).with(error_layer);

    // When DV_LOG and DV_LOG_FILE are specified and the file exists,
    // append logs into the file. The reason to append rather than
    // write logs is keeping records across the whole compilation.
    let reg = if let Some(log_file) = env::var(ENV_LOG)
        .map(|log| {
            log.parse::<log::Level>().ok()?;
            log_file()
        })
        .ok()
        .flatten()
    {
        let fmt_layer = fmt_layer.with_writer(log_file);
        reg.with(fmt_layer).try_init()
    } else {
        reg.with(fmt_layer).try_init()
    };

    if let Err(err) = reg {
        eprintln!("Logger already init: {err}");
    };

    color_eyre::install().unwrap();
}

fn log_file() -> Option<fs::File> {
    let file = env::var(ENV_LOG_FILE).ok()?;
    fs::OpenOptions::new().append(true).open(file).ok()
}
