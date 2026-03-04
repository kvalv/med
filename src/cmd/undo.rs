use crate::app::App;

pub fn undo(app: &mut App) -> Result<(), String> {
    app.buf.undo();
    Ok(())
}
