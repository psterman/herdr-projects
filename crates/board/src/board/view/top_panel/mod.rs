//! The panel on top of the list: the charts, in the order the setting says, each on a line of its own. What a chart
//! says is in the chart's own file; what a key does to the panel is in update/top_panel.rs. The row of marks above
//! them says which charts there are and which are on, and a click on a mark turns that chart on or off.
pub mod caps;
pub mod marks;
pub mod menu;
pub mod spark;
pub mod states;
pub mod waits;

use crate::board::state::State;
use crate::shown::ink::Inked;

/// The panel's lines: the row of marks, then the charts in the order the setting says, as many as fit in `h` lines,
/// the rest left out. Where each of them was drawn is kept, for a click.
pub fn lines(s: &mut State, w: usize, h: usize) -> Vec<Inked> {
    s.top_panel.hits.clear();
    s.top_panel.mark_hits.clear();
    if h == 0 { return vec![]; }
    let charts = crate::board::update::top_panel::charts(s);
    let mut out = vec![];
    // The row of marks: every chart the panel knows, and where each of them stands for a click.
    let (row, hits) = marks::row(w);
    for (_, from, to, id) in hits { s.top_panel.mark_hits.push((out.len(), from, to, id.to_string())); }
    out.push(row);
    for id in charts {
        if out.len() >= h { break; }
        let (line, hits) = match id {
            "states" => states::row(s, w),
            "activity" => (spark::row(&s.top_panel.numbers.days.iter().map(|(n, _)| *n).collect::<Vec<_>>(), "new", w), vec![]),
            "finished" => (spark::row(&s.top_panel.numbers.days.iter().map(|(_, f)| *f).collect::<Vec<_>>(), "done", w), vec![]),
            "waiting" => (waits::row(s, crate::clock::now(), w), vec![]),
            "limits" => (caps::row(s, w), vec![]),
            _ => continue,
        };
        for (from, to, which) in hits { s.top_panel.hits.push((out.len(), from, to, which)); }
        out.push(line);
    }
    out
}
