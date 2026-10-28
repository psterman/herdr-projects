//! The top panel: the charts the setting says, in the order it says them, and the menu that changes it while the
//! board runs: a mark for each chart, space turns one on or off, the arrows move it, d puts the default back, and
//! what the menu leaves is saved.
use crate::common;
use common::board::{open, Board};
use common::*;
use serde_json::json;
use sw::json::*;
use sw::store::settings::change::set_value;
use sw::store::settings::lookup::Scope;

/// The rows the panel draws: between the filter row and the list.
fn panel_rows(s: &mut Board, cols: usize) -> Vec<String> {
    let lines = s.screen(cols);
    let head = lines.iter().position(|l| l.starts_with(" All ")).map(|n| n + 1).unwrap_or(1);
    let list = s.s.list_at.map(|(t, _)| t).unwrap_or(lines.len());
    lines[head..list.max(head)].to_vec()
}
/// The chart a row says, by what it says. An empty row is none: the panel keeps its room while its menu is open, and
/// a chart that is turned off leaves its row empty.
fn chart_of(row: &str) -> String {
    for (says, name) in [("Nothing on the board", "states"), ("to do", "states"), (" new", "activity"), (" done", "finished"), ("wait", "waiting"), ("cap", "limits")] {
        if row.contains(says) { return name.to_string(); }
    }
    row.to_string()
}
/// The charts the panel shows, in the order it draws them.
fn charts(s: &mut Board, cols: usize) -> Vec<String> { panel_rows(s, cols).iter().map(|r| chart_of(r)).filter(|c| !c.is_empty()).collect() }
/// The setting as it stands.
fn said() -> Value { sw::store::settings::lookup::setting("board.top", None) }

#[test]
fn the_panel_shows_the_charts_the_setting_says_in_that_order() {
    let w = world_with(Opts { repo: true, settings: json!({ "board.top": "waiting, states" }), ..Default::default() });
    w.add("Add a greeting");
    let mut s = open(&w.root, 30); s.s.side = false;
    assert_eq!(charts(&mut s, 120), vec!["waiting", "states"], "the setting says the order");
    // and what the setting says later is what the panel draws
    set_value("board.top", &json!("activity, limits"), &Scope::you()).unwrap();
    assert_eq!(charts(&mut s, 120), vec!["activity", "limits"]);
}

#[test]
fn the_default_panel_is_the_shape_of_the_board_and_how_it_grew() {
    let w = world();
    w.add("Add a greeting");
    let mut s = open(&w.root, 30); s.s.side = false;
    assert_eq!(charts(&mut s, 120), vec!["states", "activity"]);
}

#[test]
fn the_panel_is_above_the_list_and_gives_way_when_the_room_is_small() {
    let w = world();
    w.add("Add a greeting");
    let mut s = open(&w.root, 30); s.s.side = false;
    assert_eq!(panel_rows(&mut s, 120).len(), 2);
    // the panel is above the list: the list's first row is below it
    let list = s.s.list_at.map(|(t, _)| t).unwrap_or(0);
    assert!(list >= 3, "the list starts below the panel: {list}");
    // in a window too small for it, the panel gives way before the list does
    s.rows = 6;
    assert!(panel_rows(&mut s, 120).len() <= 1, "the panel gives way before the list");
    s.rows = 30;
    assert_eq!(panel_rows(&mut s, 120).len(), 2, "with room, both charts are there");
}

#[test]
fn the_menu_lists_the_charts_with_their_marks_and_the_panel_follows_at_once() {
    let w = world();
    w.add("Add a greeting");
    let mut s = open(&w.root, 30); s.s.side = false;
    s.press(&["alt+g"]);
    let rows = s.screen(120);
    assert!(rows.iter().any(|l| l.contains("Top panel")), "{rows:?}");
    assert!(rows.iter().any(|l| l.contains("● States")), "{rows:?}");
    assert!(rows.iter().any(|l| l.contains("● Activity")), "{rows:?}");
    assert!(rows.iter().any(|l| l.contains("○ Finished")), "{rows:?}");
    assert_eq!(s.screen(120).last().unwrap().trim(), "↑↓ pick a chart · space on or off · esc close the menu");
    // the panel is still there above the menu, and follows every change
    assert_eq!(charts(&mut s, 120), vec!["states", "activity"]);
    s.press(&["space"]); // the picked chart is the first one: states goes off
    assert_eq!(charts(&mut s, 120), vec!["activity"], "the chart goes off at once");
    assert!(s.screen(120).iter().any(|l| l.contains("○ States")), "the mark says it");
    s.press(&["d"]); // the default is put back
    assert_eq!(charts(&mut s, 120), vec!["states", "activity"]);
}

#[test]
fn the_arrows_move_a_chart_and_g_and_g_go_to_the_first_and_the_last() {
    let w = world();
    w.add("Add a greeting");
    let mut s = open(&w.root, 30); s.s.side = false;
    s.press(&["alt+g"]);
    assert_eq!(s.s.top_panel.menu.as_ref().map(|m| m.at), Some(0));
    s.press(&["down"]);
    assert_eq!(s.s.top_panel.menu.as_ref().map(|m| m.at), Some(1));
    s.press(&["G"]);
    assert_eq!(s.s.top_panel.menu.as_ref().map(|m| m.at), Some(4));
    s.press(&["g"]);
    assert_eq!(s.s.top_panel.menu.as_ref().map(|m| m.at), Some(0));
    // the picked chart moves to another place: the panel draws it there
    s.press(&["right"]);
    assert_eq!(charts(&mut s, 120), vec!["activity", "states"]);
    s.press(&["left"]);
    assert_eq!(charts(&mut s, 120), vec!["states", "activity"]);
}

#[test]
fn what_the_menu_leaves_is_saved_and_the_panel_keeps_it() {
    let w = world();
    w.add("Add a greeting");
    let mut s = open(&w.root, 30); s.s.side = false;
    s.press(&["alt+g"]);
    s.press(&["down", "space"]); // activity goes off
    s.press(&["esc"]);
    assert!(s.s.top_panel.menu.is_none(), "the menu is closed");
    assert_eq!(said(), json!(["states"]), "what the menu left is saved");
    assert_eq!(charts(&mut s, 120), vec!["states"]);
    // and the board keeps it when it opens again
    let mut s = open(&w.root, 30); s.s.side = false;
    assert_eq!(charts(&mut s, 120), vec!["states"]);
}

#[test]
fn a_click_on_the_bar_shows_that_state() {
    let w = world();
    w.add("Add a greeting");
    let mut s = open(&w.root, 30); s.s.side = false;
    let rows = s.screen(120);
    let row = rows.iter().position(|l| l.contains('█')).expect("the bar is drawn");
    s.click(row as u16);
    assert_eq!(s.s.show, 5, "the to do tasks are shown");
}


#[test]
fn the_menu_shows_every_chart_where_there_is_room_and_the_picked_one_where_there_is_not() {
    let w = world();
    w.add("Add a greeting");
    let mut s = open(&w.root, 30); s.s.side = false;
    s.press(&["alt+g"]);
    let rows = s.screen(120);
    assert!(rows.iter().any(|l| l.contains("● States")), "{rows:?}");
    assert!(rows.iter().any(|l| l.contains("○ Limits")), "every chart is there where there is room: {rows:?}");
    // a window too small for them all: the picked one is in sight
    s.rows = 10;
    s.press(&["G"]);
    let rows = s.screen(120);
    assert!(rows.iter().any(|l| l.contains("Limits")), "the picked chart is in sight: {rows:?}");
    assert!(!rows.iter().any(|l| l.contains("States")), "the others do not fit there: {rows:?}");
    s.press(&["g"]); // the first one
    let rows = s.screen(120);
    assert!(rows.iter().any(|l| l.contains("States")), "and the first one is in sight too: {rows:?}");
}

#[test]
fn every_chart_has_its_own_icon_in_the_menu() {
    let w = world();
    w.add("Add a greeting");
    let mut s = open(&w.root, 30); s.s.side = false;
    s.press(&["alt+g"]);
    let rows = s.screen(120);
    // the icon of each chart, and the mark that says whether it is on
    for (icon, name) in [("▤", "States"), ("↗", "Activity"), ("✓", "Finished"), ("◷", "Waiting"), ("▥", "Limits")] {
        assert!(rows.iter().any(|l| l.contains(&format!("{icon} ● {name}")) || l.contains(&format!("{icon} ○ {name}"))), "the {name} row has its icon: {rows:?}");
    }
}

#[test]
fn a_click_on_an_icon_turns_that_chart_on_or_off_and_the_panel_follows() {
    let w = world();
    w.add("Add a greeting");
    let mut s = open(&w.root, 30); s.s.side = false;
    s.press(&["alt+g"]);
    let rows = s.screen(120);
    let line = rows.iter().position(|l| l.contains("◷ ○ Waiting")).expect("the waiting chart is there") as u16;
    // the icon's cell: the same on every row, so the click finds the chart it names
    s.click_at(3, line);
    assert!(s.screen(120).iter().any(|l| l.contains("◷ ● Waiting")), "the chart is on: {:?}", s.screen(120));
    assert!(charts(&mut s, 120).contains(&"waiting".to_string()), "the panel draws it at once: {:?}", charts(&mut s, 120));
    // the menu stands where it was, so that the next click finds the chart it names
    let rows = s.screen(120);
    assert_eq!(rows.iter().position(|l| l.contains("◷ ● Waiting")), Some(line as usize), "the menu did not move: {rows:?}");
    s.click_at(3, line);
    assert!(!charts(&mut s, 120).contains(&"waiting".to_string()), "and off again: {:?}", charts(&mut s, 120));
}

#[test]
fn a_click_on_a_chart_row_picks_it_and_the_menu_stays_open() {
    let w = world();
    w.add("Add a greeting");
    let mut s = open(&w.root, 30); s.s.side = false;
    s.press(&["alt+g"]);
    let rows = s.screen(120);
    let line = rows.iter().position(|l| l.contains("● Activity")).expect("the activity chart is there") as u16;
    s.click_at(12, line); // its name, not its icon
    assert_eq!(s.s.top_panel.menu.as_ref().map(|m| m.at), Some(1), "the chart is picked");
    assert!(s.s.top_panel.menu.is_some(), "the menu stays open");
}
