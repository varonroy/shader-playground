mod cli;
mod game_window;
mod gl;
mod logging_utils;
mod plane_buffer;
mod shader;
mod shader_playground;

use clap::Parser;
use log::info;
use shader_playground::ShaderPlaygroundArgs;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    logging_utils::init_logger(cli.debug)?;

    let args = ShaderPlaygroundArgs {
        file: cli.file,
        debouncer_ms: cli.debouncer_ms,
    };

    let window = game_window::GameWindow::new(
        "Shader Playground",
        false,
        shader_playground::ShaderPlayground::new,
        args,
    );

    info!("created window");

    info!("running");

    window.run()?;

    info!("done");

    Ok(())
}
