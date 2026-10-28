//! The top panel's menu: the charts as they stand, each with its mark and its icon, and the one that is picked. The
//! panel itself stays above it and follows every change, so what is drawn is what the panel says. Where each row's
//! icon was drawn is kept, so that a click on the icon turns that chart on or off.
use crate::board::view::list::hl;
use crate::board::state::State;
use crate::board::theme::Theme;
use crate::shown::ink::{Ink, Inked};

/// The cell the icon of a row stands in, counted from the left of the window: the same on every row, so a click
/// finds it wherever the row is.
const ICON_AT: usize = 3;

/// The rows: the title, and each chart with its mark, its icon and what it shows. `room` is how many lines there are:
/// where there are fewer than all of them, the ones around the picked one are shown, so that it is always in sight.
pub fn lines(s: &mut State, t: &Theme, w: usize, room: usize) -> Vec<Inked> {
    let Some(m) = s.top_panel.menu.as_ref() else { return vec![] };
    let charts: Vec<(bool, &'static str, &'static str)> = m.charts.iter().filter_map(|(id, on)| crate::board::top_panel::chart(id).map(|(_, name, says)| (*on, name, says))).collect();
    let at = m.at;
    let col = charts.iter().map(|(_, name, _)| name.len() + 6).max().unwrap_or(10).max(12);
    let mut out = vec![Inked::plain(" ").add(Inked::plain("Top panel").bold())];
    // The charts that fit under the title, and where they start: the picked one is in sight.
    let rows = room.saturating_sub(1);
    let from = if rows == 0 { 0 } else { at.min(charts.len().saturating_sub(1)).saturating_sub(rows.saturating_sub(1)).min(charts.len().saturating_sub(rows)) };
    // The icon of every chart: where it stands is the same on every row.
    let icons: Vec<&'static str> = m.charts.iter().map(|(id, _)| crate::board::top_panel::icon(id)).collect();
    s.top_panel.menu_hits.clear();
    for (n, (on, name, says)) in charts.iter().enumerate().skip(from).take(rows) {
        let mark = if *on { "●" } else { "○" };
        let icon = icons.get(n).copied().unwrap_or("·");
        let head = || Inked::of(format!("{icon} {mark} {name}"), if *on { Ink::Text } else { Ink::Muted });
        let gap = Inked::spaces(col.saturating_sub(name.len() + 4));
        let tail = Inked::of(format!(" · {says}"), Ink::Muted);
        let line = Inked::plain("   ").add(head()).add(gap.clone()).add(tail.clone());
        let picked = Inked::plain(" ❯ ").add(head()).add(gap).add(tail);
        // The icon's cell on this row, and the chart's place in the menu: the click that turns it on or off.
        s.top_panel.menu_hits.push((out.len(), ICON_AT, ICON_AT + 1, n));
        out.push(if n == at { hl(t, &picked, w) } else { line });
    }
    out
}
