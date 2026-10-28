//! The top panel's numbers and its menu: what the panel shows, counted from the tasks in view, and the keys that
//! change which charts it shows and in what order.
use super::Effect;
use crate::board::job::Job;
use crate::board::keys::{is, Press, K};
use crate::board::state::{Caps, Numbers, State, TopMenu};
use crate::board::top_panel::{self, DEFAULT};
use crate::json::*;
use crate::shown::states::group_of;

/// The days the panel's series covers.
pub const DAYS: usize = 14;
/// From this share of a cap on, a task is near it.
pub const NEAR: f64 = 0.8;

/// The panel's numbers, counted from the tasks in view: a day of new tasks and of finished ones (oldest first), what
/// waits for you and since when, and the running tasks' use of their caps.
pub fn numbers(tasks: &[crate::board::state::Rec]) -> Numbers {
    let now = crate::clock::now();
    let today = crate::clock::day(now);
    let mut days = vec![(0usize, 0usize); DAYS];
    let mut waiting = (0usize, now);
    let mut caps = Caps::default();
    for r in tasks {
        if r.b("archivedAt") { continue; }
        let made = r.i("created");
        if made > 0 {
            let back = today - crate::clock::day(made);
            if (0..DAYS as i64).contains(&back) { days[(DAYS as i64 - 1 - back) as usize].0 += 1; }
        }
        let group = group_of(r);
        let at = r.i("at");
        if group == "done" && at > 0 {
            let back = today - crate::clock::day(at);
            if (0..DAYS as i64).contains(&back) { days[(DAYS as i64 - 1 - back) as usize].1 += 1; }
        }
        if group == "needs you" {
            waiting.0 += 1;
            if at > 0 && at < waiting.1 { waiting.1 = at; }
        }
        let lim = r.at("limits");
        let time = lim.at("time");
        let tokens = lim.at("tokens");
        let mut cap = |used: f64, limit: &str, what: &str| {
            if limit.is_empty() || limit == crate::store::settings::values::NONE { return; }
            let room = if what == " time" { crate::store::settings::values::ms(limit) as f64 } else { crate::store::settings::values::tokens_of(limit) as f64 };
            if room <= 0.0 { return; }
            let share = (used / room).clamp(0.0, 1.0);
            if share > caps.share { caps.share = share; caps.cap = format!("{limit}{what}"); }
            if share >= NEAR { caps.near += 1; }
        };
        cap(time.f("used"), &time.s("limit"), " time");
        cap(tokens.f("used"), &tokens.s("limit"), " tokens");
    }
    Numbers { days, waiting, caps }
}

/// The charts that are on, as they stand: the menu's own list while it is open, the setting otherwise.
pub fn charts(s: &State) -> Vec<&'static str> { match &s.top_panel.menu { Some(m) => m.on(), None => top_panel::charts() } }

/// The menu opens on the charts as they stand: the ones that are on first, in the order they are drawn, the ones that
/// are off after them, in the order the panel knows them.
pub fn open(s: &mut State) {
    let charts: Vec<(String, bool)> = top_panel::all().into_iter().map(|(id, on)| (id.to_string(), on)).collect();
    s.top_panel.menu = Some(TopMenu { at: 0, charts });
}

/// The menu closes, and what it says is saved.
pub fn close(s: &mut State, fx: &mut Vec<Effect>) {
    let Some(m) = s.top_panel.menu.take() else { return };
    let said = m.said();
    if said == top_panel::said() { return; }
    fx.push(Effect::Run(Job::Set { id: String::new(), name: top_panel::SETTING.into(), value: Some(said.join(", ")), whose: crate::board::state::Whose::You }));
    fx.push(Effect::Load);
}

/// The menu's own keys: the pick moves, a chart is turned on or off, its place is moved, and the default is put back.
pub fn key(s: &mut State, p: &Press, fx: &mut Vec<Effect>) {
    if is(p, K::Back) || is(p, K::Menu) || is(p, K::Close) { return close(s, fx); }
    let n = s.top_panel.menu.as_ref().map(|m| m.charts.len()).unwrap_or(0);
    if n == 0 { return; }
    let at = s.top_panel.menu.as_ref().map(|m| m.at).unwrap_or(0);
    if is(p, K::Up) { return pick(s, at as i64 - 1, n); }
    if is(p, K::Down) { return pick(s, at as i64 + 1, n); }
    if is(p, K::Fold) { return move_to(s, at, at.saturating_sub(1)); }
    if is(p, K::Unfold) { return move_to(s, at, (at + 1).min(n - 1)); }
    if p.ch == Some(' ') || is(p, K::Talk) { return turn(s, at); }
    if p.ch == Some('d') { return reset(s); }
    if p.ch == Some('g') { return pick(s, 0, n); }
    if p.ch == Some('G') { return pick(s, n as i64 - 1, n); }
}

/// The pick moves to that place, going round the end.
pub fn pick(s: &mut State, to: i64, n: usize) {
    if n == 0 { return; }
    if let Some(m) = s.top_panel.menu.as_mut() { m.at = to.rem_euclid(n as i64) as usize; }
}

/// The pick moves one step, where there is one to move to.
pub fn step(s: &mut State, d: i64, n: usize) {
    let at = s.top_panel.menu.as_ref().map(|m| m.at as i64).unwrap_or(0);
    pick(s, at + d, n);
}

/// The chart at that place is turned on or off. It keeps its place in the menu; the panel draws the ones that are on,
/// in the order the menu has them.
fn turn(s: &mut State, at: usize) {
    if let Some(m) = s.top_panel.menu.as_mut() { if let Some((_, on)) = m.charts.get_mut(at) { *on = !*on; } }
}

/// A click in the menu: on a chart's icon it turns that chart on or off, anywhere else on its row it picks it. Nothing
/// else is touched: the menu stays open, and the panel above it follows at once.
pub fn click(s: &mut State, x: usize, y: usize) {
    let Some((_, from, to, at)) = s.top_panel.menu_hits.iter().find(|h| h.0 == y).copied() else { return };
    if x >= from && x < to { return turn(s, at); }
    pick(s, at as i64, s.top_panel.menu.as_ref().map(|m| m.charts.len()).unwrap_or(0));
}

/// A click on the panel's own row of marks: that chart is turned on or off, and what the panel shows is saved at once
/// (there is no menu to save it later).
pub fn click_mark(s: &mut State, x: usize, y: usize, fx: &mut Vec<Effect>) {
    let Some((_, _, _, id)) = s.top_panel.mark_hits.iter().find(|h| h.0 == y && x >= h.1 && x < h.2).cloned() else { return };
    let mut on = top_panel::charts();
    if on.contains(&id.as_str()) { on.retain(|c| c != &id); } else { on.push(id.as_str()); }
    let said: Vec<String> = on.into_iter().map(String::from).collect();
    fx.push(Effect::Run(Job::Set { id: String::new(), name: top_panel::SETTING.into(), value: Some(said.join(", ")), whose: crate::board::state::Whose::You }));
    fx.push(Effect::Load);
}

/// The chart at that place moves to another place in the menu: the panel draws it there.
fn move_to(s: &mut State, at: usize, to: usize) {
    let Some(m) = s.top_panel.menu.as_mut() else { return };
    if at >= m.charts.len() || to >= m.charts.len() || at == to { return; }
    let chart = m.charts.remove(at);
    m.charts.insert(to, chart);
    m.at = to;
}

/// The default: the shape of the board, and how it grew.
fn reset(s: &mut State) {
    let Some(m) = s.top_panel.menu.as_mut() else { return };
    for (id, lit) in m.charts.iter_mut() { *lit = DEFAULT.contains(&id.as_str()); }
    m.at = 0;
}
