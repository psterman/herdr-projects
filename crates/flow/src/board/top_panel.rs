//! The board's top panel (`board.top`): the charts it draws, in the order it draws them, and what each one shows.
//! The setting says which charts are on and in what order; the panel's own menu changes it while the board runs.
use crate::json::*;
use crate::store::settings::lookup::setting;

pub const SETTING: &str = "board.top";
/// The charts the panel knows, in the order the menu offers them: (id, its name, what it shows).
pub const CHARTS: [(&str, &str, &str); 5] = [
    ("states", "States", "what the board holds, by state"),
    ("activity", "Activity", "new tasks a day, for 14 days"),
    ("finished", "Finished", "tasks that reached done, a day"),
    ("waiting", "Waiting", "how long the oldest of them has waited"),
    ("limits", "Limits", "time and tokens against the cap"),
];
/// The charts that are on when nothing is said: the shape of the board, and how it grew.
pub const DEFAULT: [&str; 2] = ["states", "activity"];
/// The chart of that id, if it is one of ours.
pub fn chart(id: &str) -> Option<(&'static str, &'static str, &'static str)> { CHARTS.iter().find(|(c, ..)| *c == id).copied() }
/// The mark of each chart, the one the menu draws beside it and a click takes: what the chart shows, in one cell.
pub fn icon(id: &str) -> &'static str {
    match id {
        "states" => "▤",   // what the board holds, by state: a bar of cells
        "activity" => "↗", // new tasks a day: rising
        "finished" => "✓",  // tasks that reached done
        "waiting" => "◷",  // how long the oldest of them has waited: a clock
        "limits" => "▥",   // what the running tasks have used of their caps: a bar of a cap
        _ => "·",
    }
}
/// The charts that are on, in the order they are drawn: the ones you named, in your order. A name that is not a chart
/// is left out, and a chart named twice is said once.
pub fn charts() -> Vec<&'static str> {
    let said = setting(SETTING, None);
    let names: Vec<String> = match &said {
        Value::Array(a) => a.iter().map(text).collect(),
        Value::String(s) => s.split(',').map(|x| x.trim().to_string()).collect(),
        _ => vec![],
    };
    let mut out: Vec<&'static str> = vec![];
    for name in names {
        if let Some((id, ..)) = chart(&name) { if !out.contains(&id) { out.push(id); } }
    }
    out
}
/// The charts that are on, as the setting says them: their ids in the order they are drawn.
pub fn said() -> Vec<String> { charts().into_iter().map(String::from).collect() }
/// Every chart the panel knows, with whether it is on: the ones that are on first, in the order they are drawn, the
/// ones that are off after them, in the order the panel knows them. The menu's list and the panel's own row of marks
/// are the same list.
pub fn all() -> Vec<(&'static str, bool)> {
    let on = charts();
    let mut out: Vec<(&'static str, bool)> = on.iter().map(|id| (*id, true)).collect();
    for (id, ..) in CHARTS.iter() { if !on.contains(id) { out.push((id, false)); } }
    out
}
