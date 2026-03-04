use log::info;

use crate::{
    app::App,
    buffer::history::Change,
    cmd::pattern::Motion,
    textobject::{Boundary, TextObject},
};

pub fn delete(app: &mut App) -> Result<(), String> {
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
            let span = app.buf.span(motion);
            let change = Change {
                span: span.clone(),
                old: app
                    .buf
                    .delete_span(span, motion.boundary != Boundary::Current),
                new: "".to_string(),
            };
            app.buf.register_change(change);
        }
        _ => {
            panic!("unknown motion from '{}'", app.cmdbuf.text());
            app.buf.d(count, Boundary::Current, TextObject::Word);
            info!("Deleted {} word(s)", count);
        }
    }
    Ok(())
}
