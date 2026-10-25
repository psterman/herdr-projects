//! What the board holds, by state: one bar, each state in its own colour, and the counts of the states that have
//! something in them. Where the row is too narrow the least telling count goes first; the bar keeps its room.
use crate::board::state::State;
use crate::board::update::shows::NEEDS;
use crate::shown::ink::{Ink, Inked};
use crate::shown::states::group_ink;

/// The states in the bar, in the order they need you: (the filter that counts it, its name, the group it is in).
const STATES: [(usize, &str, &str); 5] =
    [(NEEDS, "needs you", "needs you"), (4, "ready", "ready to approve"), (3, "working", "working"), (5, "to do", "to do"), (6, "done", "done")];
/// The bar's least width, so that each state can be seen in it, and its most.
const BAR_MIN: usize = 12;
const BAR_MAX: usize = 24;

/// The row, and where each state landed in it: (first cell, last cell, the filter that shows it).
pub fn row(s: &mut State, w: usize) -> (Inked, Vec<(usize, usize, usize)>) {
    let room = w.saturating_sub(2);
    let all: Vec<(usize, usize, &str, &str)> = STATES.iter().filter_map(|(n, name, group)| {
        let c = s.show_n.get(*n).copied().unwrap_or(0);
        if c > 0 { Some((*n, c, *name, *group)) } else { None }
    }).collect();
    if all.is_empty() { return (Inked::plain(" ").add(Inked::of("Nothing on the board yet.", Ink::Muted)), vec![]); }
    // The counts, the least telling first to go where the row is too narrow: the bar keeps its room.
    let facts_room = room.saturating_sub(BAR_MIN + 2);
    let mut facts = all.clone();
    while facts.len() > 1 && width(&facts) > facts_room { facts.pop(); }
    // Where not even one count fits beside the bar, the bar stands alone.
    let (facts, bar_w) = if width(&facts) > facts_room { (vec![], room.min(BAR_MAX)) } else { let w = width(&facts); (facts, room.saturating_sub(w + 2).min(BAR_MAX).max(BAR_MIN).min(room)) };
    let (bar, hits) = bar(&all, bar_w);
    let mut out = Inked::plain(" ").add(bar);
    if !facts.is_empty() { out = out.add(Inked::plain("  ")).add(words(&facts)); }
    (out, hits)
}

/// How wide the counts are, with the gaps between them.
fn width(facts: &[(usize, usize, &str, &str)]) -> usize {
    facts.iter().enumerate().map(|(n, (_, c, name, _))| name.len() + 1 + c.to_string().len() + if n > 0 { 3 } else { 0 }).sum()
}
/// The counts, each in its own ink: the one that waits for you in red.
fn words(facts: &[(usize, usize, &str, &str)]) -> Inked {
    let mut out = Inked::new();
    for (n, (_, count, name, group)) in facts.iter().enumerate() {
        if n > 0 { out = out.add(Inked::of(" · ", Ink::Muted)); }
        out = out.add(Inked::of(format!("{name} "), Ink::Muted)).add(Inked::of(count.to_string(), if *group == "needs you" { Ink::Red } else { Ink::Soft }));
    }
    out
}
/// The bar: a cell for each part of the whole, each state in its own colour, and where each landed for a click.
fn bar(facts: &[(usize, usize, &str, &str)], width: usize) -> (Inked, Vec<(usize, usize, usize)>) {
    let total: usize = facts.iter().map(|(_, c, ..)| *c).sum();
    let mut cells: Vec<usize> = facts.iter().map(|(_, c, ..)| (*c * width) / total).collect();
    let mut used: usize = cells.iter().sum();
    // A state with something in it is seen, where there is room for it.
    for (n, (_, c, ..)) in facts.iter().enumerate() {
        if *c > 0 && cells[n] == 0 && used < width { cells[n] = 1; used += 1; }
    }
    let rest = width.saturating_sub(used);
    if rest > 0 {
        if let Some(n) = facts.iter().enumerate().max_by_key(|(_, (_, c, ..))| *c).map(|(n, _)| n) { cells[n] += rest; }
    }
    let mut out = Inked::new();
    let mut hits = vec![];
    let mut at = 1;
    for (n, (filter, _, _, group)) in facts.iter().enumerate() {
        out = out.add(Inked::of("█".repeat(cells[n]), group_ink(group)));
        hits.push((at, at + cells[n], *filter));
        at += cells[n];
    }
    (out, hits)
}
