//! The panel on top of the list: the charts, in the order the setting says, each on a line of its own. What a chart
//! says is in the chart's own file; what a key does to the panel is in update/top_panel.rs.
pub mod caps;
pub mod menu;
pub mod spark;
pub mod states;
pub mod waits;

use crate::board::state::State;
use crate::shown::ink::Inked;

/// The panel's lines, in the order the charts are: as many as fit in `h` lines, the rest left out. Where each of them
/// was drawn is kept, for a click.
pub fn lines(s: &mut State, w: usize, h: usize) -> Vec<Inked> {
    s.top_panel.hits.clear();
    let charts = crate::board::update::top_panel::charts(s);
    let mut out = vec![];
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
