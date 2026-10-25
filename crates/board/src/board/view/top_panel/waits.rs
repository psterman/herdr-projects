//! What waits for you, and how long the oldest of them has waited: the things the board holds that it cannot go on
//! with until you say so.
use crate::board::state::State;
use crate::shown::ink::{Ink, Inked};
use crate::shown::words::age;

/// The row: how many wait, and how long the oldest has waited.
pub fn row(s: &State, now: i64, w: usize) -> Inked {
    let (count, since) = s.top_panel.numbers.waiting;
    let room = w.saturating_sub(1);
    if count == 0 {
        let say = if room >= 22 { "Nothing waits for you." } else { "Nothing waits." };
        return Inked::plain(" ").add(Inked::of(say, Ink::Muted));
    }
    let who = if count == 1 { "waits for you" } else { "wait for you" };
    let mut out = Inked::plain(" ").add(Inked::of(count.to_string(), Ink::Red)).add(Inked::of(format!(" {who}"), Ink::Text));
    // How long the oldest has waited, where there is room for it: it is the first to go.
    let old = format!(" · the oldest {}", age(now - since));
    if out.width() + old.len() <= room { out = out.add(Inked::of(old, Ink::Muted)); }
    out
}
