//! The line at the bottom of the board: what it holds and who works on it, in numbers, not what the keys do.
use crate::board::state::State;
use crate::json::*;
use crate::shown::ink::{Ink, Inked};

/// The line at the bottom: the agent the chat talks to and how it stands, what the board holds, the time. What does not
/// fit is let go, the least telling first: the model's name, then the counts, so the agent, the state and the time are
/// always there.
pub fn status_line(s: &State, width: usize) -> Inked {
    let room = width.saturating_sub(2);
    let state = state_of(s);
    let clock = crate::clock::hm(crate::clock::now());
    let mut counts = counts_of(s);
    loop {
        let text = line(&model_of(s), &state, &counts, &clock);
        if crate::text::width(&text) <= room { return Inked::plain("  ").add(Inked::of(text, Ink::Muted)); }
        if !counts.is_empty() { counts.pop(); continue; }
        // The model's name goes: what it is called, what it does and the time stay.
        let text = line(&kind_of(s), &state, &counts, &clock);
        if crate::text::width(&text) <= room { return Inked::plain("  ").add(Inked::of(text, Ink::Muted)); }
        return Inked::plain("  ").add(Inked::of(text, Ink::Muted)).cut(width);
    }
}
/// The line's parts, joined by a dot.
fn line(model: &str, state: &str, counts: &[String], clock: &str) -> String {
    let mut parts: Vec<String> = vec![];
    if !model.is_empty() { parts.push(model.to_string()); }
    if !state.is_empty() { parts.push(state.to_string()); }
    parts.extend(counts.iter().cloned());
    parts.push(clock.to_string());
    parts.join(" · ")
}
/// The program and the model the chat talks to, as the board writes them.
fn model_of(s: &State) -> String {
    let Some(k) = s.data.coord.run.as_ref().or(s.data.coord.def.as_ref()) else { return String::new() };
    let kind = k.s("kind");
    if kind.is_empty() { return String::new(); }
    if k.b("model") { format!("{kind} · {}", k.s("model")) } else { kind.to_string() }
}
/// The program the chat talks to, without the model's name.
fn kind_of(s: &State) -> String {
    s.data.coord.run.as_ref().or(s.data.coord.def.as_ref()).map(|k| k.s("kind").to_string()).unwrap_or_default()
}
/// How it stands, in one word.
fn state_of(s: &State) -> String {
    let state = s.data.coord.state.as_str();
    let word = match state { "" => "", "none" => "coordinator off", "starting" => "starting", "working" => "working", "blocked" => "needs you", "idle" => "idle", "exited" => "stopped", other => other };
    if s.data.coord.queue > 0 { return format!("{word} · {} waiting", s.data.coord.queue); }
    word.to_string()
}
/// What the board holds: the tasks it has, and the ones that want something.
fn counts_of(s: &State) -> Vec<String> {
    let c = crate::board::update::shows::counts(s, &s.items);
    let at = |n: usize| c.get(n).copied().unwrap_or(0);
    let mut out = vec![plural(at(0), "task")];
    for (n, word) in [(2usize, "need you"), (3, "working"), (4, "ready"), (6, "done")] {
        if at(n) > 0 { out.push(format!("{} {word}", at(n))); }
    }
    if at(7) > 0 { out.push(plural(at(7), "automation")); }
    out
}
/// One thing and how many there are of it: "1 task", "3 tasks".
fn plural(n: usize, word: &str) -> String { if n == 1 { format!("{n} {word}") } else { format!("{n} {word}s") } }
