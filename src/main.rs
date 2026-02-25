use clap::Parser;
use std::path::PathBuf;

use crate::app::App;

pub mod app;
pub mod cmdbuf;
pub mod event;
pub mod ui;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    file: PathBuf,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let terminal = ratatui::init();
    let result = App::new(&args.file).run(terminal).await;
    ratatui::restore();
    result
}
