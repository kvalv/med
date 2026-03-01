use crate::{
    app::{App, Mode},
    event::AppEvent,
};

pub fn append(app: &mut App) -> Result<(), String> {
    match app.cmdbuf.pop_left() {
        Some('a') => app.buf.right(1),
        Some('A') => app.buf.eol(),
        other => return Err(format!("Unexpected '{:?}'", other)),
    }

    app.events.send(AppEvent::ModeChange(Mode::Insert));
    Ok(())
}
