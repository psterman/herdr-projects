//! What a chat shows of a conversation: only its words, whoever said them. The readers for each agent program (Claude
//! Code, Codex, pi), the words on a screen for a program that cannot be read, and markdown read as text in the box.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use sw::agents::screen_text::screen_text;
use sw::agents::talk::talk_of;

fn jsonl(file: &Path, rows: &[Value]) -> PathBuf { std::fs::create_dir_all(file.parent().unwrap()).unwrap(); std::fs::write(file, rows.iter().map(|r| r.to_string()).collect::<Vec<_>>().join("\n") + "\n").unwrap(); file.to_path_buf() }
fn ts(n: u32) -> String { format!("2026-10-13T12:00:{n:02}.000Z") }
fn claude_rows(prompt: &str, said: &[&str]) -> Vec<Value> {
    let mut v = vec![json!({ "type": "user", "timestamp": ts(1), "message": { "role": "user", "content": prompt } })];
    for (n, text) in said.iter().enumerate() {
        let n = n as u32;
        v.push(json!({ "type": "assistant", "timestamp": ts(2 + n * 3), "message": { "id": format!("m{n}"), "role": "assistant", "content": [{ "type": "text", "text": text }] } }));
        v.push(json!({ "type": "assistant", "timestamp": ts(3 + n * 3), "message": { "id": format!("m{n}"), "role": "assistant", "content": [{ "type": "tool_use", "id": format!("t{n}"), "name": "Bash", "input": { "command": "npm run check -- --all" } }] } }));
        v.push(json!({ "type": "user", "timestamp": ts(4 + n * 3), "message": { "role": "user", "content": [{ "type": "tool_result", "tool_use_id": format!("t{n}"), "content": "76 test files passed" }] } }));
    }
    v
}
fn words(kind: &str, file: &Path) -> Vec<String> { talk_of(kind, Some(file), "", Path::new("/"), 50).unwrap_or_default().iter().map(|m| format!("{}: {}", if m.you { "you" } else { "them" }, m.text)).collect() }

#[test]
fn a_saved_conversation_is_read_as_its_words_alone_for_every_program() {
    let dir = std::env::temp_dir().join(format!("sw-talk-{}", std::process::id()));
    let mut rows = claude_rows("Fix the greeting", &["I will read the file first."]);
    rows.extend([
        json!({ "type": "user", "isMeta": true, "timestamp": ts(20), "message": { "role": "user", "content": "[Image: original 2560x1600]" } }),
        json!({ "type": "user", "timestamp": ts(21), "version": "2.2.0", "promptId": "p1", "message": { "role": "user", "content": "<local-command-stdout>ok</local-command-stdout>" } }),
        json!({ "type": "user", "timestamp": ts(21), "version": "2.2.0", "promptId": "p2", "promptSource": "queue", "origin": { "kind": "task-notification" }, "message": { "role": "user", "content": "<task-notification>a helper finished</task-notification>" } }),
        json!({ "type": "user", "timestamp": ts(21), "version": "2.2.0", "promptId": "p3", "promptSource": "typed", "origin": { "kind": "human" }, "message": { "role": "user", "content": "<question>Is it done?</question>" } }),
        json!({ "type": "user", "timestamp": ts(21), "version": "2.2.0", "origin": { "kind": "human" }, "message": { "role": "user", "content": "and the tests?" } }),
        json!({ "type": "user", "timestamp": ts(21), "version": "2.2.0", "promptSource": "typed", "message": { "role": "user", "content": "<note>no origin named</note>" } }),
        json!({ "type": "assistant", "isSidechain": true, "timestamp": ts(22), "message": { "id": "side", "role": "assistant", "content": [{ "type": "text", "text": "a helper's words" }] } }),
        json!({ "type": "assistant", "timestamp": ts(23), "message": { "id": "m9", "role": "assistant", "content": [{ "type": "thinking", "thinking": "hm" }] } }),
        json!({ "type": "assistant", "timestamp": ts(24), "message": { "id": "m9", "role": "assistant", "content": [{ "type": "text", "text": "Fixed." }] } }),
        json!({ "type": "assistant", "timestamp": ts(25), "message": { "id": "m9", "role": "assistant", "content": [{ "type": "text", "text": "It says hello now." }] } }),
    ]);
    let claude = jsonl(&dir.join("claude.jsonl"), &rows);
    assert_eq!(words("claude", &claude), vec!["you: Fix the greeting", "them: I will read the file first.", "you: <question>Is it done?</question>", "you: and the tests?", "you: <note>no origin named</note>", "them: Fixed.\n\nIt says hello now."]);
    let mut plain = claude_rows("<question>Ready?</question>", &["Ready to review."]);
    plain.push(json!({ "type": "user", "timestamp": ts(30), "promptId": "p9", "message": { "role": "user", "content": "ordinary words" } }));
    assert_eq!(words("claude", &jsonl(&dir.join("plain.jsonl"), &plain)), vec!["you: <question>Ready?</question>", "them: Ready to review.", "you: ordinary words"]);
    // words typed while it works are queued, and written down as they go in: a person's are yours, nothing else is
    let queued = |a: Value| json!({ "type": "attachment", "timestamp": ts(31), "attachment": a });
    plain.extend([
        queued(json!({ "type": "queued_command", "commandMode": "prompt", "origin": { "kind": "human" }, "prompt": "and the tests too" })),
        queued(json!({ "type": "queued_command", "commandMode": "task-notification", "origin": { "kind": "task-notification" }, "prompt": "<task-notification>a helper finished</task-notification>" })),
        queued(json!({ "type": "queued_command", "commandMode": "prompt", "origin": { "kind": "peer" }, "prompt": "another session's words" })),
        json!({ "type": "attachment", "isMeta": true, "timestamp": ts(31), "attachment": { "type": "queued_command", "origin": { "kind": "coordinator" }, "prompt": "a reminder" } }),
        queued(json!({ "type": "queued_command", "commandMode": "prompt", "prompt": [{ "type": "text", "text": "an older version names no origin" }] })),
        queued(json!({ "type": "edited_text_file", "filename": "f.txt" })),
        json!({ "type": "assistant", "timestamp": ts(32), "message": { "id": "m7", "role": "assistant", "content": [{ "type": "text", "text": "Both done." }] } }),
    ]);
    assert_eq!(words("claude", &jsonl(&dir.join("queued.jsonl"), &plain))[2..], ["you: ordinary words", "you: and the tests too", "you: an older version names no origin", "them: Both done."]);
    let item = |ty: &str, text: &str| json!({ "timestamp": ts(30), "type": "event_msg", "payload": { "type": "item_completed", "item": { "type": ty, "content": [{ "type": "text", "text": text }] } } });
    let codex = jsonl(&dir.join("codex.jsonl"), &[
        json!({ "timestamp": ts(30), "type": "event_msg", "payload": { "type": "task_started" } }),
        json!({ "timestamp": ts(30), "type": "response_item", "payload": { "type": "message", "role": "developer", "content": [{ "type": "input_text", "text": "<skills_instructions>" }] } }),
        json!({ "timestamp": ts(30), "type": "response_item", "payload": { "type": "message", "role": "user", "content": [{ "type": "input_text", "text": "# AGENTS.md instructions for /repo" }] } }),
        json!({ "timestamp": ts(30), "type": "response_item", "payload": { "type": "message", "role": "user", "content": [{ "type": "input_text", "text": "Review the change" }] } }),
        item("UserMessage", "Review the change"),
        json!({ "timestamp": ts(31), "type": "response_item", "payload": { "type": "message", "role": "assistant", "phase": "commentary", "content": [{ "type": "output_text", "text": "I will run the full check.\n" }] } }),
        item("AgentMessage", "I will run the full check.\n"),
        json!({ "timestamp": ts(32), "type": "response_item", "payload": { "type": "custom_tool_call", "name": "exec", "input": "npm run check" } }),
        json!({ "timestamp": ts(33), "type": "response_item", "payload": { "type": "custom_tool_call_output", "output": [{ "type": "input_text", "text": "Script completed" }] } }),
        json!({ "timestamp": ts(34), "type": "response_item", "payload": { "type": "reasoning", "summary": [] } }),
        json!({ "timestamp": ts(35), "type": "response_item", "payload": { "type": "message", "role": "assistant", "phase": "final_answer", "content": [{ "type": "output_text", "text": "All 76 test files pass." }] } }),
    ]);
    assert_eq!(words("codex", &codex), vec!["you: Review the change", "them: I will run the full check.", "them: All 76 test files pass."]);
    let older = jsonl(&dir.join("older.jsonl"), &[json!({ "timestamp": ts(40), "type": "response_item", "payload": { "type": "message", "role": "user", "content": [{ "type": "input_text", "text": "Do it" }] } }),
        json!({ "timestamp": ts(41), "type": "response_item", "payload": { "type": "message", "role": "assistant", "content": [{ "type": "output_text", "text": "Done." }] } })]);
    assert_eq!(words("codex", &older), vec!["you: Do it", "them: Done."]);
    let pi = jsonl(&dir.join("pi.jsonl"), &[json!({ "type": "session", "id": "s" }), json!({ "type": "message", "timestamp": ts(50), "message": { "role": "system", "content": "You are pi" } }),
        json!({ "type": "message", "timestamp": ts(51), "message": { "role": "user", "content": [{ "type": "text", "text": "Say hello" }] } }),
        json!({ "type": "message", "timestamp": ts(52), "message": { "role": "assistant", "content": [{ "type": "toolCall", "name": "bash" }, { "type": "text", "text": "hello" }] } }),
        json!({ "type": "message", "timestamp": ts(53), "message": { "role": "toolResult", "content": [{ "type": "text", "text": "exit 0" }] } })]);
    assert_eq!(words("pi", &pi), vec!["you: Say hello", "them: hello"]);
    assert!(talk_of("other", Some(&pi), "", Path::new("/"), 50).is_none());
    assert!(talk_of("claude", Some(&dir.join("none.jsonl")), "", Path::new("/"), 50).is_none());
    use std::io::Write;
    std::fs::OpenOptions::new().append(true).open(&pi).unwrap().write_all(format!("{}\n", json!({ "type": "message", "timestamp": ts(54), "message": { "role": "assistant", "content": [{ "type": "text", "text": "anything else?" }] } })).as_bytes()).unwrap();
    assert_eq!(talk_of("pi", Some(&pi), "", Path::new("/"), 1).unwrap().iter().map(|m| m.text.clone()).collect::<Vec<_>>(), vec!["anything else?"]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_screens_words_leave_out_what_is_drawn_and_markdown_reads_as_text() {
    let splash = [">_ OpenAI Codex (v0.162.1)", "    ~/work", "", "        ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿", "      ⣿⣿⣿⣿⣿  ⣿⣿⣿⣿⣿", "", "╭──────────────────╮", "│ Which colour?    │", "│ ❯ 1. Blue        │", "╰──────────────────╯", "────────────────────", "▁▂▃▄▅▆▇█ 80%"].join("\n");
    assert_eq!(screen_text(&splash), ">_ OpenAI Codex (v0.162.1)\n    ~/work\n\nWhich colour?\n❯ 1. Blue");
    assert_eq!(screen_text(""), ""); assert_eq!(screen_text("⣿⣿⣿⣿\n────"), "");
    let lines: Vec<String> = sw::board::view::chat_text::prose_lines("## Result\n**Pass.** All of it.\n- a point that is long enough to wrap\n```js\nconst a = 1;\n```\n1. first", 22, false).iter().map(|l| l.text()).collect();
    assert_eq!(lines, vec!["Result", "Pass. All of it.", "- a point that is long", "  enough to wrap", "  const a = 1;", "1. first"]);
}

#[test]
fn a_screen_cut_at_its_own_width_is_put_back_into_the_lines_it_was_one() {
    // What a program wrote on a screen of its own width: its words are cut in two at the edge.
    let screen = ["╭─ ☤ Hermes ────────────────────────────────╮",
        "Actually — maybe the user is testing. The most sensible response: ask what \"1\" refers",
        "to, briefly. But maybe I should consider whether there was a numbered list of options somewhere. My",
        " last reply had no numbered options. So \"1\" is likely just a stray input or a poke to test responsiv",
        "eness.",
        "",
        "Let me check if there's any pending question from the board — maybe an agent asked a qu",
        "estion, or the board has notifications. Let me check `board list` — no tasks. So not",
        "hing pending.",
        "",
        "Wait — actually, maybe I should reconsider: could \"1\" refer to picking an option f",
        "rom a previous board interaction? Let me check the board's other coordinator state or the herdr tran",
        "script. Let me look for any pending \"pick\" state: state.json \"pick\": null.",
        "",
        "「1」我看不出指向什么——板上没有任务。",
        "☤ MiniMax-M3.1-Flash │ ~42.9K/1M │ ~4%",
        "❯ Set a reminder or schedule a recurring task"].join("\n");
    let text = sw::board::view::unfold::prose(&screen_text(&screen));
    assert!(text.contains("ask what \"1\" refers to, briefly"), "a break at a space keeps its space: {text}");
    assert!(text.contains("My last reply had no numbered options"), "and the one that moved it too: {text}");
    assert!(text.contains("to test responsiveness"), "the word cut in two is whole: {text}");
    assert!(text.contains("an agent asked a question, or the board has notifications"), "and the next one: {text}");
    assert!(text.contains("So nothing pending."), "the sentence stands whole: {text}");
    assert!(text.contains("picking an option from a previous board interaction"), "and this one: {text}");
    assert!(text.contains("the herdr transcript"), "the last word of the screen too: {text}");
    assert!(text.contains("「1」我看不出指向什么——板上没有任务。"), "the words of another tongue stay: {text}");
    assert!(text.contains("MiniMax-M3.1-Flash"), "the bar of status stands: {text}");
    assert!(text.contains("Set a reminder"), "and the box of the program: {text}");
    assert!(!text.contains("Hermes"), "the frame it drew around its words is left out: {text}");
}

#[test]
fn a_hermes_conversation_is_read_from_the_store_it_keeps() {
    use rusqlite::Connection;
    let dir = std::env::temp_dir().join(format!("sw-hermes-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("state.db");
    {
        let con = Connection::open(&db).unwrap();
        con.execute_batch("create table sessions (id text primary key, model text, input_tokens integer, output_tokens integer);
            create table messages (id integer primary key, session_id text, role text, content text, timestamp real, tool_name text, finish_reason text);").unwrap();
        con.execute("insert into sessions values ('s1', 'MiniMax-M3.1-Flash', 100, 20)", []).unwrap();
        // a prompt, a call of its own, the tool's words, and the answer
        for (id, role, content, ts, why) in [(1i64, "user", "Fix the greeting", 1.0, ""), (2, "assistant", "", 2.0, "tool_calls"),
            (3, "tool", "the file has three lines", 3.0, ""), (4, "assistant", "Fixed.", 4.0, "stop")] {
            con.execute("insert into messages values (?1, 's1', ?2, ?3, ?4, '', ?5)", rusqlite::params![id, role, content, ts, why]).unwrap();
        }
    }
    unsafe { std::env::set_var("HERMES_HOME", &dir); }
    let rows = sw::agents::hermes_store::rows_in(&db, "s1").unwrap();
    assert_eq!(rows.len(), 4);
    let said = sw::agents::hermes_store::talk(&rows);
    assert_eq!(said.iter().map(|m| format!("{}: {}", if m.you { "you" } else { "them" }, m.text)).collect::<Vec<_>>(), vec!["you: Fix the greeting", "them: Fixed."]);
    let turn = sw::agents::hermes_store::turn(&rows).unwrap();
    assert!(turn.finished && !turn.pending, "the answer ends the turn: {turn:?}");
    assert_eq!(turn.said, "Fixed.");
    assert_eq!(turn.id, "4");
    assert_eq!(turn.began, 1000, "the prompt began the turn: {turn:?}");
    let turn = sw::agents::hermes_store::turn_in(&db, "s1").unwrap();
    assert_eq!(turn.model, "MiniMax-M3.1-Flash");
    assert_eq!(turn.usage.unwrap().out, 20.0);
    assert_eq!(sw::agents::hermes_store::holds_in(&db, "s1", "Fix the greeting", 0), Some(true));
    assert_eq!(sw::agents::hermes_store::holds_in(&db, "s1", "Fix the greeting", 5000), Some(false), "it was written before that moment");
    assert_eq!(sw::agents::hermes_store::holds_in(&db, "s1", "nothing of it", 0), Some(false));
    assert!(sw::agents::hermes_store::rows_in(&db, "other").unwrap().is_empty());
    // The board's own ways in: the words of a chat, and the end of a turn.
    let words = talk_of("hermes", None, "s1", Path::new("/"), 50).unwrap();
    assert_eq!(words.iter().map(|m| format!("{}: {}", if m.you { "you" } else { "them" }, m.text)).collect::<Vec<_>>(), vec!["you: Fix the greeting", "them: Fixed."]);
    let turn = sw::agents::installed::read_turn_of("hermes", None, "s1", Path::new("/"), false).unwrap();
    assert!(turn.finished && turn.said == "Fixed.", "the end of the turn is read from the store: {turn:?}");
    assert_eq!(sw::agents::installed::said_to("hermes", None, "s1", Path::new("/"), "Fix the greeting", 0), Some(true));
    assert!(sw::agents::installed::read_turn_of("hermes", None, "none", Path::new("/"), false).is_none(), "a conversation that is not there is nothing to read");
    let _ = std::fs::remove_dir_all(&dir);
}

use crate::common;
use common::*;
use sw::board::keys::key_name;
use sw::json::*;

fn chat_box(s: &mut common::board::Board) -> (String, Vec<String>) { let c = crate::board_chat::chat(s, 120); let lines = s.screen(120); let top = lines.iter().rposition(|l| l.starts_with("╭─")).unwrap(); (lines[top].clone(), c.boxed) }

#[test]
fn a_review_under_way_shows_what_the_reviewer_says_never_its_screen_and_a_working_agents_words_never_its_tool_calls() {
    let w = world_with(Opts { repo: true, settings: json!({ "reviewer.agent": "claude" }), ..Default::default() });
    let claude_dir = std::path::PathBuf::from(std::env::var("CLAUDE_CONFIG_DIR").unwrap());
    let splash = ">_ OpenAI Codex (v0.162.1)\n\n   ⣿⣿⣿⣿⣿⣿⣿⣿\n\n› Ask Codex to do anything\n  ? for shortcuts\n";
    w.script(json!({ "worker": [{ "files": { "f.txt": "1\n" }, "commit": "1", "note": "Made f." }] }));
    let id = to_approval(&w, "Make f");
    // A review under way, as the review runner leaves it while its reviewer works: a first review, so nothing of an
    // earlier one (its verdict, the change's size) is on the task yet (it waits for the verdict on a
    // thread of its own, which no test runs): the reviewer's pane, its program's start screen, its conversation's id.
    let api = sw::herdr::Herdr::new(None);
    let ws = task(&id).at("link").t("workspace");
    let pane = api.tab_create(&ws, "", "Reviewer #1", &[]).unwrap().at("root_pane").t("pane_id");
    api.agent_start("sw-proj-1-r", "claude", &pane, &[], 1000).unwrap();
    w.patch(|st| { for p in st["panes"].as_array_mut().unwrap() { if p.s("pane_id") == pane { p["agent"]["status"] = json!("working"); p["agent"]["screen"] = json!(splash); } } });
    sw::store::tasks::update(&id, |t| { t.put("stage", "reviewing"); t.put("msg", ""); t.put("reviewSession", "rev-1"); t.put("reviewRound", 1); for k in ["review", "reviews", "stat", "pr"] { t.shift_remove(k); } t.obj_mut("link").put("reviewer", json!({ "pane": pane, "kind": "claude" })); Ok(sw::store::tasks::Change::Write) }).unwrap();
    let t = task(&id);
    let mut s = common::board::open(&w.root, 34); s.s.side = false;
    s.s.sel = id.clone(); s.load(); s.press(&["space"]);
    let (_, b) = chat_box(&mut s);
    assert_eq!(b, vec!["The reviewer has just started: nothing to show yet.", "", "❯ Send message to the agent: the review goes on unless the code changes"]);
    assert_eq!(crate::board_chat::chat_kind(&mut s, 120), "Reviewer", "the box's edge names the reviewer as whom it talks to");
    assert!(!regex::Regex::new("OpenAI Codex|Ask Codex|for shortcuts|⣿|Made f\\.").unwrap().is_match(&s.text()), "{}", s.text());
    let file = jsonl(&claude_dir.join("projects").join("x").join(format!("{}.jsonl", t.s("reviewSession"))), &claude_rows("Work board task proj/1, role: reviewer, round 1.\n\nYou are an adversarial reviewer.", &["I will check the earlier faults first.", "The full check passes. Now the browser."]));
    s.load();
    let (edge, b) = chat_box(&mut s);
    assert_eq!(b, vec!["Reviewer:", "The full check passes. Now the browser.", "", "❯ Send message to the agent: the review goes on unless the code changes"]);
    assert!(edge.contains(&format!("{} more ↑", key_name("chatScrollUp"))), "{edge}");
    assert!(!regex::Regex::new("Bash|npm run check|76 test files|OpenAI Codex").unwrap().is_match(&s.text()));
    s.press(&["alt+k"]);
    let (_, b) = chat_box(&mut s);
    assert_eq!(b[..b.len() - 2].to_vec(), vec!["Board:", "Work board task proj/1, role: reviewer, round 1.", "You are an adversarial reviewer.", "", "Reviewer:", "I will check the earlier faults first.", "", "Reviewer:", "The full check passes. Now the browser."]);
    s.press(&["alt+j"]);
    let (_, b) = chat_box(&mut s); assert_eq!(b[..b.len() - 2].to_vec(), vec!["Reviewer:", "The full check passes. Now the browser."]);
    use std::io::Write;
    std::fs::OpenOptions::new().append(true).open(&file).unwrap().write_all(format!("{}\n", json!({ "type": "assistant", "timestamp": ts(40), "message": { "id": "m7", "role": "assistant", "content": [{ "type": "text", "text": "One fault left: the header is cut." }] } })).as_bytes()).unwrap();
    s.load();
    let (_, b) = chat_box(&mut s); assert_eq!(b[..2].to_vec(), vec!["Reviewer:", "One fault left: the header is cut."]);
    s.press(&["space"]); s.s.side = true;
    let wide = s.screen(160).join("\n");
    assert!(regex::Regex::new(r"The reviewer says ─+ *\n[^\n]*One fault left: the header is cut\.").unwrap().is_match(&wide) && !regex::Regex::new("The agent said|On its screen").unwrap().is_match(&wide), "{wide}");
    s.press(&["space"]); s.s.side = false;
    // another machine gets the same: the conversation travels with the task
    let sent = sw::remote::remote_list().at("tasks").as_array().unwrap().iter().find(|x| x.s("id") == id).unwrap().at("snap").at("talk").clone();
    assert_eq!(sent.s("who"), "reviewer"); assert_eq!(sent.arr("list").last().unwrap().s("text"), "One fault left: the header is cut.");
    // the review ends: the chat is the agent's again
    sw::store::tasks::update(&id, |t| { t.put("stage", "approval"); t.obj_mut("link").put("reviewer", Value::Null); Ok(sw::store::tasks::Change::Write) }).unwrap(); s.load();
    let (_, b) = chat_box(&mut s);
    assert_eq!(b, vec!["Made f.", "", "❯ Send message to the agent"]);
}
