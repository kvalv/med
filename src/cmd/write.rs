use crate::app::App;

// #[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
// pub struct BufWrite {}

pub fn buf_write(app: &mut App) -> Result<(), String> {
    std::fs::write(&app.filename, app.buf.text()).expect("Failed to write file");
    Ok(())
}

// impl CmdHandler for BufWrite {
//     fn handle(&self, app: &mut App) {
//         std::fs::write(&app.filename, app.buf.text()).expect("Failed to write file");
//     }
// }
