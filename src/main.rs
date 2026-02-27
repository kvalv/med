use clap::Parser;
use std::path::PathBuf;

use crate::app::App;

pub mod app;
pub mod buffer;
pub mod cmdbuf;
pub mod event;
pub mod ui;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                // "[{} {} {}] {}",
                // humantime::format_rfc3339_seconds(SystemTime::now()),
                // record.level(),
                "{}: {}",
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        // .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    file: PathBuf,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    setup_logger()?;

    color_eyre::install()?;
    let args = Args::parse();

    let terminal = ratatui::init();
    let result = App::new(&args.file).run(terminal).await;
    ratatui::restore();
    result
}
