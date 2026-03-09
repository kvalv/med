use log::info;

use crate::{app::App, cmd::Movement};

pub fn movement(app: &mut App, movement: &Movement) -> Result<(), String> {
    // app.buf.w(m.count);
    let target_pos = movement.span(&mut app.buf).end;
    app.buf.position(target_pos.row, target_pos.col);

    let (should_update, target_col) = should_update_target_col(movement);
    if should_update {
        app.buf.target_col = target_col.or(Some(target_pos.col));
    }

    info!("Executing movement: {:?}", movement);
    Ok(())
}

fn should_update_target_col(movement: &Movement) -> (bool, Option<usize>) {
    // For horizontal movements, we want to update the target column
    // for $ we want 9999
    match movement.char {
        '$' => (true, Some(9999)),
        'h' | 'l' | '0' | 'w' => (true, None),
        _ => (false, None),
    }
}
