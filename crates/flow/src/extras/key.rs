//! The key that opens the board in herdr, changed while the board runs: the plugin's own block in herdr's config.toml
//! is written with that key (the old one is taken out first), the file is checked on a copy before it is written, the
//! old one is kept as a backup, and herdr is told to reload. A key herdr or the config already uses is refused, with
//! what that key does today.
use super::config::{accepted, block_text, herdr_config, read, reload};
use std::path::PathBuf;

/// The setting that says which key opens the board in herdr.
pub const BOARD_KEY: &str = "board.key";

/// The plugin's block for the key that opens the board, taken out of the text: None when it was not there.
fn remove(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"\n?# >>> herdr-projects key[^\n]*\n(?s:.*?)# <<< herdr-projects key[^\n]*\n?").ok()?;
    if !regex::Regex::new(r"(?m)^# >>> herdr-projects key").unwrap().is_match(text) { return None; }
    Some(re.replace_all(text, "\n").to_string())
}
/// The file with the plugin's key block taken out, and the new one put in its place (or added, when there was none).
fn with_key(text: &str, chord: &str) -> String {
    let out = remove(text).unwrap_or_else(|| text.to_string());
    let out = out.trim_end();
    let out = if out.is_empty() { String::new() } else { format!("{out}\n") };
    format!("{out}\n{}\n", block_text("key", chord))
}
/// The key that opens the board is this one: herdr's config is written with it, and herdr is told to reload. Ok(what
/// was said) or Err(why it was not).
pub fn set(chord: &str) -> Result<String, String> {
    let file = herdr_config();
    let text = read();
    // A key herdr itself uses, or one the config already binds, is never taken: it is said what it does today.
    let defs = crate::herdr::keys::defaults();
    crate::herdr::keys::check(chord, &text, &defs, &crate::store::marks::tilde(&file.display().to_string())).map_err(|m| m)?;
    let next = with_key(&text, chord);
    // The file is checked on a copy before it is written: herdr's answer is asked of the copy, never of the real one.
    accepted(&next).map_err(|why| format!("herdr would reject that: {why}"))?;
    let backup = PathBuf::from(format!("{}.bak-herdr-projects", file.display()));
    let existed = file.exists();
    let wrote = (|| -> std::io::Result<()> {
        if let Some(d) = file.parent() { std::fs::create_dir_all(d)?; }
        if existed { std::fs::copy(&file, &backup)?; }
        std::fs::write(&file, next)
    })();
    if wrote.is_err() { return Err(format!("Could not write {}.", crate::store::marks::tilde(&file.display().to_string()))); }
    let (status, why) = reload();
    if status == "failed" {
        if existed { let _ = std::fs::copy(&backup, &file); } else { let _ = std::fs::remove_file(&file); }
        reload();
        return Err(format!("herdr rejected the change, so your settings were put back as they were. It said: {}", crate::store::tasks::one_line(&why, 300)));
    }
    Ok(if status == "unknown" { format!("Added. herdr could not be asked to reload ({}): restart herdr or run herdr server reload-config.", crate::store::tasks::one_line(&why, 120)) } else { format!("{chord} now opens the board.") })
}
