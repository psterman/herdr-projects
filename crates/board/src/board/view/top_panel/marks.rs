//! The panel's own row of marks: every chart the panel knows, one icon each, the ones that are on in herdr's text and
//! the ones that are off in the faint grey. The row is there as long as the panel is, and a click on a mark turns that
//! chart on or off. The panel below draws the ones that are on, in the order the row shows them.
use crate::shown::ink::{Ink, Inked};

/// The cell the first mark stands in, counted from the left of the window.
pub const AT: usize = 1;
/// The marks of every chart, in the order the panel knows them: the ones that are on first, in the order they are
/// drawn, the ones that are off after them. Where each of them was drawn is kept, for a click.
pub fn row(w: usize) -> (Inked, Vec<(usize, usize, usize, &'static str)>) {
    let charts = crate::board::top_panel::all();
    let mut out = Inked::plain(" ");
    let mut hits = vec![];
    let mut at = AT;
    for (id, on) in charts {
        let icon = crate::board::top_panel::icon(id);
        out = out.add(Inked::of(icon, if on { Ink::Text } else { Ink::Muted }));
        hits.push((0, at, at + 1, id));
        at += 2;
        out = out.add(Inked::plain(" ")); // a space between the marks, so that each is a cell of its own
    }
    (out.cut(w), hits)
}
