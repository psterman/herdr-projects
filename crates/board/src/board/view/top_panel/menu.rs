//! The top panel's menu: the charts as they stand, each with its mark, and the one that is picked. The panel itself
//! stays above it and follows every change, so what is drawn is what the panel says.
use crate::board::view::list::hl;
use crate::board::state::State;
use crate::board::theme::Theme;
use crate::shown::ink::{Ink, Inked};

/// The rows: the title, and each chart with its mark and what it shows.
pub fn lines(s: &mut State, t: &Theme, w: usize) -> Vec<Inked> {
    let Some(m) = s.top_panel.menu.as_ref() else { return vec![] };
    let charts: Vec<(bool, &'static str, &'static str)> = m.charts.iter().filter_map(|(id, on)| crate::board::top_panel::chart(id).map(|(_, name, says)| (*on, name, says))).collect();
    let at = m.at;
    let col = charts.iter().map(|(_, name, _)| name.len() + 4).max().unwrap_or(8).max(10);
    let mut out = vec![Inked::plain(" ").add(Inked::plain("Top panel").bold())];
    for (n, (on, name, says)) in charts.iter().enumerate() {
        let mark = if *on { "●" } else { "○" };
        let head = || Inked::of(format!("{mark} {name}"), if *on { Ink::Text } else { Ink::Muted });
        let gap = Inked::spaces(col.saturating_sub(name.len() + 2));
        let tail = Inked::of(format!(" · {says}"), Ink::Muted);
        let line = Inked::plain(" ").add(head()).add(gap.clone()).add(tail.clone());
        out.push(if n == at { hl(t, &Inked::plain(" ").add(Inked::of("❯", Ink::Accent)).add(Inked::plain(" ")).add(head()).add(gap).add(tail), w) } else { line });
    }
    out
}
