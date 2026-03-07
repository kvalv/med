use log::info;

use crate::{app::App, textobject::Motion};

pub fn movement(app: &mut App) -> Result<(), String> {
    // 4w for example
    let (motion, _) = Motion::from_cmd(&app.cmdbuf.text());
    info!("motion is {:?}", &motion);
    match motion {
        Some(motion) => {
            let span = app.buf.span(motion);
            info!("span is {}", &span);
            app.buf.position(span.end.row, span.end.col - 1);
            app.buf.update_target_col();
            Ok(())
        }
        _ => Err("No motion found for change".to_string()),
    }
}
