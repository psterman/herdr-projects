//! The conversation on its own, whole: what the agent you talk to has said, as far back as the pane goes, instead of the
//! list of tasks. Nothing is around it: no head, no frame, no rule, no empty line. Up and down scroll it, and what is
//! out of view is said in one greyed line, above or under.
use super::chat::Who;
use super::talk::Part;
use crate::board::state::State;
use crate::shown::ink::{Ink, Inked};

/// The conversation, wrapped to `width`, newest at the bottom, and nothing else around it.
pub fn conversation(s: &mut State, who: Option<&Who>, width: usize, h: usize) -> Vec<Inked> {
    let Some(w) = who else { return vec![Inked::of("Nobody to talk to.".to_string(), Ink::Muted)] };
    if w.parts.is_empty() {
        s.panel_height = 1;
        s.panel_length = 1;
        s.panel_top = 0;
        return vec![];
    }
    // Your words are as bright as you type them, theirs grey, and the screen of a program that cannot be read is fainter
    // still: in the open, the words stand out from the program they came from.
    let paint = |by: bool, l: Inked, p: &Part| if by { l.recolor(Ink::Muted) } else if !p.tone.is_empty() { l.recolor(crate::board::update::beat::ink_of(&p.tone)) } else if p.mine || p.by == "You" { l.recolor(Ink::Plain) } else if p.raw { l.recolor(Ink::Muted) } else { l.recolor(Ink::Soft) };
    let room = width.saturating_sub(2).max(1);
    let (lines, _starts) = super::chat_text::part_lines(&w.parts, room, &paint);
    let lines: Vec<Inked> = lines.into_iter().map(|l| Inked::plain("  ").add(l)).collect();
    // The pane shows as many lines as it has room for, and keeps the end in sight while new words come. One row may
    // say there is more, above; another, under: the words keep the rest of the pane.
    let total = lines.len();
    let was_end = s.panel_top.saturating_add(s.panel_height) >= s.panel_length;
    let mut height = h.max(1);
    let (mut top, mut end) = (0, 0);
    for _ in 0..4 {
        let last = total.saturating_sub(height);
        top = if was_end { last } else { s.panel_top.min(last) };
        end = total.min(top + height);
        let says = usize::from(top > 0) + usize::from(end < total);
        if height + says <= h || height == 1 { break; }
        height -= 1;
    }
    s.panel_height = height;
    s.panel_length = total;
    s.panel_top = top;
    let mut out: Vec<Inked> = lines[top..end].to_vec();
    // What is out of view is said in one greyed line, above or under: the words keep the rest of the pane.
    if top > 0 && out.len() < h { out.insert(0, Inked::of(format!("  ↑ {} more", top), Ink::Muted)); }
    if end < total && out.len() < h { out.push(Inked::of(format!("  ↓ {} more", total - end), Ink::Muted)); }
    out
}
