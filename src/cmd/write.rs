use crate::{app::App, cmd::CmdHandler};

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct BufWrite {}

impl CmdHandler for BufWrite {
    fn handle(&self, app: &mut App) {
        std::fs::write(&app.filename, app.buf.text()).expect("Failed to write file");
    }
}

impl std::fmt::Display for BufWrite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "w")
    }
}
