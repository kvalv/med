use log::info;

use crate::app::App;

pub fn movement(app: &mut App) -> Result<(), String> {
    let count = app.cmdbuf.pop_count().unwrap_or(1);

    info!(
        "Movement {} word(s) - rest of buf '{}'",
        count,
        app.cmdbuf.text()
    );

    match app.cmdbuf.pop() {
        Some('w') => app.buf.w(count),
        Some('b') => app.buf.b(count),
        _ => return Ok(()),
    }

    app.buf.update_target_col();
    Ok(())
}
