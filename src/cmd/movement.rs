use crate::app::App;

pub fn movement(app: &mut App) -> Result<(), String> {
    let count = app.cmdbuf.pop_count(1);
    let c = app.cmdbuf.pop().expect("c is none");

    match c {
        'w' => app.buf.w(count),
        'e' => app.buf.e(count),
        _ => todo!(),
    }
    app.buf.update_target_col();

    Ok(())
}
