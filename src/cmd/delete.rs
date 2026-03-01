use log::info;

use crate::{
    app::App,
    cmd::{CmdHandler, pattern::Motion},
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
        info!("cmdhandler.delete: text is '{}'", app.cmdbuf.text());

        let count = app.cmdbuf.pop_count().unwrap_or(1);
        // now we'll expect that 'd' is the argument

        match app.cmdbuf.pop_left() {
            Some('d') => {}
            x => panic!("Expected 'd' but it's {:?}", x),
        }

        let (motion, _) = Motion::from_cmd(&app.cmdbuf.text());
        info!("got motion {:?}", motion);

        match motion {
            Some(motion) => {
                info!(
                    "Deleted count={:?} boundary={:?} object={:?}",
                    &motion.count.unwrap_or(1),
                    &motion.boundary,
                    &motion.object
                );
                app.buf
                    .d(motion.count.unwrap_or(1), motion.boundary, motion.object);
            }
            _ => {
                app.buf.d(count, Boundary::Current, TextObject::Word);
                info!("Deleted {} word(s)", count);
            }
        }
    }
}
