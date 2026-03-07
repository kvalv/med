use log::info;

use crate::{
    app::App,
    buffer::history::Change,
    textobject::{Boundary, parse_textobject},
};

pub fn delete(app: &mut App) -> Result<(), String> {
    info!("cmdhandler.delete: text is '{}'", app.cmdbuf.text());

    let count = app.cmdbuf.pop_count(1);
    // now we'll expect that 'd' is the argument

    match app.cmdbuf.pop_left() {
        Some('d') => {}
        x => panic!("Expected 'd' but it's {:?}", x),
    }

    let (parsed, _) = parse_textobject(&app.cmdbuf.text());
    info!("got textobject {:?}", parsed);

    match parsed {
        Some((object, boundary, obj_count)) => {
            let count = obj_count.unwrap_or(count);
            info!(
                "Deleted count={:?} boundary={:?} object={:?}",
                count, &boundary, &object
            );
            let span = app.buf.span_for_textobject(object, boundary, count);
            let change = Change {
                span,
                old: app.buf.delete_span(span, boundary != Boundary::Current),
                new: "".to_string(),
            };
            app.buf.register_change(change);
        }
        _ => {
            panic!("unknown motion from '{}'", app.cmdbuf.text());
        }
    }
    Ok(())
}
