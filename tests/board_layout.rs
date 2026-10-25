//! The whole screen: the chat block (the text box, which names who you talk to, what just happened, the hints) stands
//! right under the list, and the hints are the last line, at any height, through a resize, while the block grows or shrinks.
use crate::common;
use common::board::{open, Board};
use common::*;
use serde_json::json;

fn pinned(s: &mut Board, rows: usize, what: &str) -> (usize, usize) {
    s.rows = rows;
    let lines = s.screen(120);
    let note = format!("{what}, {rows} rows");
    assert_eq!(lines.len(), rows, "{note}");
    let last = lines.len() - 1;
    let said = if s.s.msg.is_empty() { 0 } else { 1 };
    assert!(regex::Regex::new(r"^[ ▌] \S.*( · |all keys)").unwrap().is_match(&lines[last]), "{note}: the hints are the last line: {:?}", lines[last]);
    let rule = lines.iter().rposition(|l| regex::Regex::new(r"^(─+|╰─+╯)$").unwrap().is_match(l)).unwrap();
    assert_eq!(rule, last - 1 - said, "{note}: the text box sits right on the hints");
    // The chat block starts with the text box's top edge.
    let head = lines.iter().position(|l| regex::Regex::new(r"^(─+|╭─.*╮)$").unwrap().is_match(l)).unwrap_or_else(|| panic!("{note}: no text box\n{}", lines.join("\n")));
    if rows >= 16 { assert_eq!(lines[head - 1].trim(), "", "{note}: an empty line sets the chat off from the list"); }
    for l in &lines[head..=last] { assert_ne!(l.trim(), "", "{note}: an empty line inside the chat block"); }
    let tasks = lines[..head].iter().filter(|l| l.contains("Task number")).count();
    // the list's own rows above the tasks: the machine's row, its group, the line that counts the rest, the empty line
    let list = s.s.list_at.unwrap().0;
    if rows >= 16 { assert!(tasks + 4 >= head - list, "{note}: {tasks} tasks in {head} lines"); }
    (head, tasks)
}

#[test]
fn the_chat_is_pinned_to_the_bottom_tall_resized_and_while_it_grows_or_shrinks() {
    let w = world();
    for i in 1..=230 { w.add(&format!("Task number {i}")); }
    let mut s = open(&w.root, 84); s.s.side = false;
    let tall = pinned(&mut s, 84, "tall");
    assert!(tall.1 > 70);
    for rows in [120, 200, 61, 84, 40, 24, 16, 12, 9, 58, 84] { pinned(&mut s, rows, "resized"); }
    assert_eq!(pinned(&mut s, 84, "back to tall"), tall);
    s.typed(&"word ".repeat(60));
    let draft = pinned(&mut s, 84, "wrapped draft");
    assert!(draft.0 < tall.0, "the box grows upwards");
    for rows in [200, 20, 84] { pinned(&mut s, rows, "wrapped draft, resized"); }
    s.s.flash("Something just happened.", sw::shown::ink::Ink::Yellow);
    pinned(&mut s, 84, "with a message"); pinned(&mut s, 140, "with a message, resized");
    s.s.msg.clear(); s.s.input.set_text("");
    assert_eq!(pinned(&mut s, 84, "shrunk back"), tall);
}

#[test]
fn the_line_at_the_bottom_can_say_what_the_board_holds_instead_of_the_keys() {
    let w = world();
    let mut s = open(&w.root, 40);
    s.s.side = false;
    // The keys, as they have always been.
    let keys = s.screen(120).last().cloned().unwrap_or_default();
    assert!(keys.contains("all keys"), "the keys are the last line: {keys}");
    // The data, when the setting says so.
    sw::store::settings::change::set_value("board.statusLine", &json!("data"), &sw::store::settings::lookup::Scope::you()).unwrap();
    let line = s.screen(120).last().cloned().unwrap_or_default();
    let line = line.trim().to_string();
    assert!(!line.contains("all keys"), "the keys give way to the data: {line}");
    assert!(line.contains("0 tasks"), "what the board holds: {line}");
    assert!(regex::Regex::new(r"\d\d:\d\d$").unwrap().is_match(&line), "and the time of day: {line}");
}

#[test]
fn the_board_can_open_on_the_conversation_alone_and_the_list_is_one_key_away() {
    let w = world_with(Opts { repo: true, settings: json!({ "board.view": "chat" }), ..Default::default() });
    w.add("Task number 1");
    let mut s = open(&w.root, 40);
    // The setting is yours: the pane shows the conversation, not the list.
    assert!(s.s.chat_whole, "the board opens on the conversation");
    let lines = s.screen(120);
    assert!(!lines.iter().any(|l| l.contains("Task number")), "the list is not there: {lines:?}");
    assert!(lines.iter().any(|l| l.contains('❯')), "the line you type on is there: {lines:?}");
    // The key brings the list back, for as long as the board runs.
    s.press(&["alt+w"]);
    assert!(!s.s.chat_whole);
    assert!(s.screen(120).iter().any(|l| l.contains("Task number")), "the list is back");
}
