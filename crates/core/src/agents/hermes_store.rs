//! Hermes keeps every conversation it ever had in a store of its own (a SQLite database under its home), not in a
//! file of lines: the board reads a session's words from there, so that a Hermes agent's chat reads like Claude
//! Code's, Codex's and pi's. Read only, and soft: a store that is locked, moved or of another shape is nothing to
//! show, and the board falls back to the words on the screen.
use super::homes::hermes_home;
use super::read_turn::{when, Turn, Usage};
use super::talk::{add, tail_of, Said};
use crate::json::*;
use rusqlite::{params, Connection, OpenFlags};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

/// Where Hermes keeps its conversations.
pub fn db_path() -> PathBuf { hermes_home().join("state.db") }
/// The store, opened for reading. A store that is locked, gone or of another shape is nothing to show.
fn open(db: &Path) -> Option<Connection> {
    for flags in [OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX, OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX] {
        if let Ok(c) = Connection::open_with_flags(db, flags) { return Some(c); }
    }
    None
}
/// The messages of one session, oldest first: {id, role, content, timestamp (ms), tool_name, finish_reason}.
pub fn rows_in(db: &Path, session: &str) -> Option<Vec<Value>> {
    let con = open(db)?;
    let mut st = con.prepare("select id, role, content, timestamp, tool_name, finish_reason from messages where session_id = ?1 order by id").ok()?;
    let rows = st.query_map([session], |r| {
        Ok(json!({ "id": r.get::<_, i64>(0)?, "role": r.get::<_, String>(1)?, "content": r.get::<_, Option<String>>(2)?.unwrap_or_default(),
            "timestamp": (r.get::<_, Option<f64>>(3)?.unwrap_or(0.0) * 1000.0) as i64, "tool_name": r.get::<_, Option<String>>(4)?.unwrap_or_default(),
            "finish_reason": r.get::<_, Option<String>>(5)?.unwrap_or_default() }))
    }).ok()?;
    Some(rows.flatten().collect())
}
/// A mark of the conversation as it is now: its newest message and how many there are. It changes when it grows.
pub fn mark_in(db: &Path, session: &str) -> Option<(i64, i64)> {
    let con = open(db)?;
    con.query_row("select coalesce(max(id), 0), count(*) from messages where session_id = ?1", [session], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))).ok()
}
/// What the store knows of the session itself: the model it ran on and the tokens it used.
fn session_in(db: &Path, session: &str) -> Option<(String, Option<Usage>)> {
    let con = open(db)?;
    let (model, inp, o) = con.query_row("select model, coalesce(input_tokens, 0), coalesce(output_tokens, 0) from sessions where id = ?1", [session],
        |r| Ok((r.get::<_, Option<String>>(0)?.unwrap_or_default(), r.get::<_, i64>(1)?, r.get::<_, i64>(2)?))).ok()?;
    let usage = (inp > 0 || o > 0).then(|| Usage { inp: inp as f64, out: o as f64, context: (inp + o) as f64, window: None });
    Some((model, usage))
}
/// The words of a Hermes conversation: what was said to it and what it said back. A tool's own words are not said.
pub fn talk(rows: &[Value]) -> Vec<Said> {
    let mut out = vec![];
    for e in rows {
        let role = e.s("role");
        if role != "user" && role != "assistant" { continue; }
        let text = e.t("content").trim().to_string();
        if text.is_empty() { continue; }
        add(&mut out, role == "user", &text, when(e.at("timestamp")));
    }
    out
}
/// The end of a Hermes conversation: the words it said last, and whether it is still at work.
pub fn turn(rows: &[Value]) -> Option<Turn> {
    let (mut id, mut said, mut began, mut pos) = (String::new(), String::new(), 0, 0);
    let (mut stop, mut finished, mut pending, mut any) = (String::new(), false, false, false);
    for e in rows {
        let role = e.s("role").to_string();
        let text = e.t("content").trim().to_string();
        let at = when(e.at("timestamp"));
        any = true;
        pos = at;
        if role == "user" {
            if !text.is_empty() { began = at; id = e.i("id").to_string(); }
            pending = true; finished = false; stop = String::new();
        } else if role == "assistant" {
            if !text.is_empty() { said = text; id = e.i("id").to_string(); }
            let why = e.s("finish_reason");
            if ["stop", "length", "refusal"].contains(&why) { finished = true; pending = false; stop = if why == "stop" { "stop".into() } else { why.into() }; }
            else { pending = true; finished = false; stop = String::new(); }
        } else { pending = true; finished = false; stop = String::new(); } // a tool's own words: it is still at work
    }
    if !any { return None; }
    Some(Turn { id, pos, began, said, stop, finished, pending, ..Default::default() })
}
/// The end of the conversation in the store, with what the store knows of the session itself.
pub fn turn_in(db: &Path, session: &str) -> Option<Turn> {
    let rows = rows_in(db, session)?;
    let (model, usage) = session_in(db, session).unwrap_or_default();
    let mut t = turn(&rows)?;
    t.model = model;
    t.usage = usage;
    Some(t)
}
/// Do these words stand in the conversation, written since a moment? The longest line of the text stands for it.
pub fn holds_in(db: &Path, session: &str, text: &str, since: i64) -> Option<bool> {
    let mut lines: Vec<&str> = text.split('\n').map(|l| l.trim()).collect();
    lines.sort_by_key(|l| std::cmp::Reverse(l.chars().count()));
    let marker: String = lines.first().map(|l| l.chars().take(80).collect()).unwrap_or_default();
    if marker.is_empty() { return Some(false); }
    let con = open(db)?;
    let hit = con.query_row("select 1 from messages where session_id = ?1 and role = 'user' and timestamp * 1000 >= ?2 and instr(content, ?3) > 0 limit 1",
        params![session, since - 2000, marker], |r| r.get::<_, i64>(0)).ok();
    Some(hit.is_some())
}

/// A mark of this program's conversation, when it keeps it in a store of its own (Hermes).
pub fn mark_of(kind: &str, session: &str) -> Option<(i64, i64)> {
    if kind != "hermes" || session.is_empty() { return None; }
    mark_in(&db_path(), session)
}
/// The words of this program's conversation, when it keeps them in a store of its own (Hermes). Read again only when
/// the conversation grew.
static TALK: LazyLock<Mutex<HashMap<String, ((i64, i64), Vec<Said>)>>> = LazyLock::new(Default::default);
pub fn talk_of(kind: &str, session: &str, keep: usize) -> Option<Vec<Said>> {
    if kind != "hermes" || session.is_empty() { return None; }
    let db = db_path();
    let mark = mark_in(&db, session)?;
    let mut cache = TALK.lock().ok()?;
    if let Some((m, list)) = cache.get(session) { if *m == mark { return Some(tail_of(list, keep)); } }
    let list = talk(&rows_in(&db, session)?);
    if cache.len() > 50 { cache.clear(); }
    cache.insert(session.to_string(), (mark, list.clone()));
    Some(tail_of(&list, keep))
}
/// The end of this program's conversation, when it keeps it in a store of its own (Hermes).
pub fn turn_of(kind: &str, session: &str) -> Option<Turn> {
    if kind != "hermes" || session.is_empty() { return None; }
    turn_in(&db_path(), session)
}
/// Do these words stand in this program's conversation, when it keeps it in a store of its own (Hermes).
pub fn holds_of(kind: &str, session: &str, text: &str, since: i64) -> Option<bool> {
    if kind != "hermes" || session.is_empty() { return None; }
    holds_in(&db_path(), session, text, since)
}
/// Is that conversation still in the store? (Hermes reopens a conversation by its id.)
pub fn has_session(session: &str) -> bool {
    !session.is_empty() && mark_in(&db_path(), session).map(|(_, n)| n > 0).unwrap_or(false)
}
