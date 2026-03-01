use log::info;

use crate::{app::App, cmd::CmdHandler};

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Movement {}

impl std::fmt::Display for Movement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "movement")
    }
}

/// w, b, ...
impl CmdHandler for Movement {
    fn handle(&self, app: &mut App) {
        let count = app.cmdbuf.pop_count().unwrap_or(1);

        info!(
            "Movement {} word(s) - rest of buf '{}'",
            count,
            app.cmdbuf.text()
        );

        match app.cmdbuf.pop() {
            Some('w') => app.buf.w(count),
            Some('b') => app.buf.b(count),
            _ => return,
        }

        app.buf.update_target_col();
    }
}
