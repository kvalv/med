use log::info;

use crate::{
    app::App,
    cmd::CmdHandler,
    textobject::{Boundary, TextObject},
};

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Delete {}

impl std::fmt::Display for Delete {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "d")
    }
}

impl CmdHandler for Delete {
    fn handle(&self, app: &mut App) {
        let count = app.cmdbuf.pop_count().unwrap_or(1);
        app.buf.d(count, Boundary::Current, TextObject::Word);
        info!("Deleted {} word(s)", count);
    }
}
