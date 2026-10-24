//! What the board's pane shows (`board.view`): the list of tasks, or the conversation on its own.
use crate::store::settings::lookup::setting;

pub const SETTING: &str = "board.view";
/// Does the pane show the conversation instead of the list of tasks?
pub fn is_chat() -> bool { setting(SETTING, None).as_str() == Some("chat") }
