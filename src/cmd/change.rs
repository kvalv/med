use log::info;

use crate::{
    app::{App, Mode},
    buffer::history::Change,
    event::AppEvent,
    textobject::parse_textobject,
};

pub fn change(app: &mut App) -> Result<(), String> {
    match app.cmdbuf.pop_left() {
        Some('c') => {}
        x => panic!("Expected 'c' but it's {:?}", x),
    }

    // TODO: c$, c0, ...
    let (parsed, _) = parse_textobject(&app.cmdbuf.text());

    match parsed {
        Some((object, boundary, count)) => {
            info!(
                "change count={:?} boundary={:?} object={:?}",
                count.unwrap_or(1),
                &boundary,
                &object
            );

            let span = app.buf.span_for_textobject(object, boundary, 1);

            let change = Change {
                span,
                old: app.buf.delete_span(span, false),
                new: "".to_string(),
            };
            // how do we 'merge' with the inserted text that is not yet figured out?
            // we'll deal with that later...

            app.buf.register_change(change);

            app.events.send(AppEvent::ModeChange(Mode::Insert));
            Ok(())
        }
        _ => Err("No motion found for change".to_string()),
    }
}
