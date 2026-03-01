use crate::{app::Mode, cmd::CmdHandler, event::AppEvent};

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Insert {}

impl CmdHandler for Insert {
    fn handle(&self, app: &mut crate::app::App) {
        app.events.send(AppEvent::ModeChange(Mode::Insert));
    }
}
