//! A series of a day at a time: one cell for each day, as high as that day was, and what the days came to. Where the
//! row is too narrow the count goes first: the shape of the days is what the row is for.
use crate::shown::ink::{Ink, Inked};

/// The heights a day is drawn at, from the least to the most.
const HEIGHTS: [&str; 8] = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
/// The fewest days a series keeps, where there is room for them.
const FEWEST: usize = 3;

/// The row: the days as cells, and what they came to.
pub fn row(days: &[usize], label: &str, w: usize) -> Inked {
    let total: usize = days.iter().sum();
    let say = format!("{total} {label}");
    let room = w.saturating_sub(2);
    // Where the count does not fit beside the days, the days keep the room and the count is left out.
    let (say, cells) = if say.len() + 2 + FEWEST <= room {
        let cells = room.saturating_sub(say.len() + 2).min(days.len());
        (say, cells)
    } else {
        (String::new(), room.min(days.len()))
    };
    let cells = cells.max(FEWEST.min(days.len())).min(room).max(1);
    let shown = &days[days.len() - cells..];
    let max = shown.iter().copied().max().unwrap_or(0);
    let mut line = Inked::new();
    for count in shown {
        let h = if max == 0 { 0 } else { ((*count as f64 / max as f64) * 7.0).round() as usize }.min(7);
        line = line.add(Inked::of(HEIGHTS[h], Ink::Muted));
    }
    let mut out = Inked::plain(" ").add(line);
    if !say.is_empty() { out = out.add(Inked::plain("  ")).add(Inked::of(say, Ink::Soft)); }
    out
}
