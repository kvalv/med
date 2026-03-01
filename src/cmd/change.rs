use log::info;

use crate::{
    app::{App, Mode},
    cmd::pattern::Motion,
    event::AppEvent,
};

pub fn change(app: &mut App) -> Result<(), String> {
    match app.cmdbuf.pop_left() {
        Some('c') => {}
        x => panic!("Expected 'c' but it's {:?}", x),
    }

    // TODO: c$, c0, ...
    let (motion, _) = Motion::from_cmd(&app.cmdbuf.text());

    match motion {
        Some(motion) => {
            info!(
                "change count={:?} boundary={:?} object={:?}",
                &motion.count.unwrap_or(1),
                &motion.boundary,
                &motion.object
            );

            app.buf
                .d(motion.count.unwrap_or(1), motion.boundary, motion.object);
            app.events.send(AppEvent::ModeChange(Mode::Insert));
            Ok(())
        }
        _ => Err("No motion found for change".to_string()),
    }
}
