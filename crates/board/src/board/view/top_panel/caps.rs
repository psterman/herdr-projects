//! What the running tasks have used of their caps: the most used of them, the cap it was, and how many are near
//! theirs. A task with no cap says nothing here.
use crate::board::state::State;
use crate::shown::ink::{Ink, Inked};
use crate::shown::words::bar;

/// The row: a bar of the cap that is most used, what part of it that is, and how many are near theirs.
pub fn row(s: &State, w: usize) -> Inked {
    let caps = &s.top_panel.numbers.caps;
    let room = w.saturating_sub(1);
    if caps.cap.is_empty() { return Inked::plain(" ").add(Inked::of("No task runs with a cap.", Ink::Muted)); }
    let pct = (caps.share * 100.0).round();
    let ink = if caps.near > 0 { Ink::Yellow } else { Ink::Muted };
    let near = if caps.near > 0 { format!(" · {} near", caps.near) } else { String::new() };
    let mut out = Inked::new();
    // The bar first, where there is room for it beside the words.
    if 8 + 2 + pct.to_string().len() + 4 + caps.cap.len() + near.len() <= room {
        out = out.add(Inked::of(bar(pct, 8), ink)).add(Inked::plain("  "));
    }
    out = out.add(Inked::of(format!("{pct:.0}% "), Ink::Soft)).add(Inked::of(caps.cap.clone(), Ink::Text));
    if !near.is_empty() && out.width() + near.len() <= room { out = out.add(Inked::of(near, Ink::Muted)); }
    out
}
