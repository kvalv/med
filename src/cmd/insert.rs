use crate::{
    app::{App, Mode},
    event::AppEvent,
};

pub fn insert(app: &mut App) -> Result<(), String> {
    app.events.send(AppEvent::ModeChange(Mode::Insert));
    Ok(())
}
