use anyhow::Context;
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

pub fn init_logger(force_debug: bool) -> anyhow::Result<()> {
    TermLogger::init(
        if cfg!(debug_assertions) || force_debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        },
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .with_context(|| "initializing logger")?;

    Ok(())
}
