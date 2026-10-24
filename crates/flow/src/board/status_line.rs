//! What the line at the bottom of the board says (`board.statusLine`): the keys that matter now, or what the board holds
//! and who works on it, in numbers.
use crate::store::settings::lookup::setting;

pub const SETTING: &str = "board.statusLine";
/// Does the line at the bottom show the data instead of the keys?
pub fn is_data() -> bool { setting(SETTING, None).as_str() == Some("data") }
