//! A program's screen, read as the words it wrote: the lines it cut at its own width are put back into the lines they
//! were one, so the text reads as prose and not as a column of half-words. What it drew for itself (a rule, a frame, a
//! bar of status) stands as it is, and the frames around its words are left out.
use crate::shown::ink::Inked;

/// The marks a program draws with: the edges of its boxes, the rules between, the bar of its status.
const MARKS: &[char] = &['─', '│', '╭', '╮', '╰', '╯', '┌', '┐', '└', '┘', '├', '┤', '┬', '┴', '┼', '━', '┃', '═', '║', '╔', '╗', '╚', '╝'];

/// How much of the line is drawn and not written: a frame or a rule, which is not words.
fn frame(line: &str) -> bool {
    let marks: Vec<char> = line.chars().filter(|c| !c.is_whitespace()).collect();
    if marks.is_empty() { return false; }
    let drawn = marks.iter().filter(|c| MARKS.contains(c)).count();
    drawn * 2 >= marks.len()
}

/// A line that stands on its own: what the program drew, or what it listed, or what it indented.
fn own(line: &str) -> bool {
    if line.chars().any(|c| MARKS.contains(&c)) { return true; } // it drew a mark of its own
    let t = line.trim_start();
    if t.starts_with(['-', '*', '+', '•', '#', '>', '❯', '»']) { return true; }
    if t.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) && t[1..].starts_with(['.', ')']) { return true; }
    line.starts_with("  ") // indented: code, or a block of its own
}

/// A line that ends a sentence: it was not cut in two, and the words after it begin another one.
fn ends_a_sentence(line: &str) -> bool {
    let Some(last) = line.trim_end().chars().last() else { return false; };
    matches!(last, '.' | '!' | '?' | '。' | '！' | '？' | '…')
}

/// The words between two lines of a screen that was cut at its own width: none when the word was cut in two, a space
/// when the break was at one. `width` is how wide the lines of their block go.
fn between(prev: &str, cur: &str, width: usize) -> &'static str {
    if cur.starts_with(char::is_whitespace) { return " "; } // the space was moved to this line
    let p = prev.trim_end();
    let Some(last) = p.chars().last() else { return " " };
    let Some(first) = cur.trim_start().chars().next() else { return " " };
    if !last.is_alphanumeric() { return " "; } // the break was at a space
    if first.is_uppercase() { return " "; }    // a new word starts here
    // The word was cut in two: what is left of it on the line before is short, or that line was full.
    let frag = p.split_whitespace().last().unwrap_or("");
    if frag.chars().count() <= 4 { return ""; }
    if crate::text::width(p) + 1 >= width { return ""; }
    " "
}

/// The lines of one block of a screen, put back into the paragraphs they were cut into.
fn join(block: &[String]) -> Vec<String> {
    // How wide the words of this block go: the width they were cut at.
    let width = block.iter().filter(|l| !own(l)).map(|l| crate::text::width(l)).max().unwrap_or(0);
    let mut out: Vec<String> = vec![];
    let mut cur: Option<String> = None;
    for line in block {
        if own(line) {
            if let Some(p) = cur.take() { out.push(p); }
            if !frame(line) { out.push(line.trim_start().to_string()); }
            continue;
        }
        match cur.take() {
            None => cur = Some(line.trim_start().to_string()),
            Some(prev) => {
                // A line that ends a sentence was not cut in two: what follows is another one.
                if ends_a_sentence(&prev) {
                    out.push(prev);
                    cur = Some(line.trim_start().to_string());
                } else {
                    let sep = between(&prev, line, width);
                    cur = Some(format!("{prev}{sep}{}", line.trim_start()));
                }
            }
        }
    }
    if let Some(p) = cur.take() { out.push(p); }
    out
}

/// The screen as prose: the lines it cut at its own width put back together, and the frames it drew left out.
pub fn prose(text: &str) -> String {
    let mut out: Vec<String> = vec![];
    let mut block: Vec<String> = vec![];
    for raw in text.split('\n') {
        let cleaned = crate::shown::words::clean(raw);
        let line = cleaned.trim_end();
        if line.trim().is_empty() {
            let paras = join(&block);
            if !paras.is_empty() {
                if !out.is_empty() { out.push(String::new()); }
                out.extend(paras);
            }
            block.clear();
            continue;
        }
        block.push(line.to_string());
    }
    let paras = join(&block);
    if !paras.is_empty() {
        if !out.is_empty() { out.push(String::new()); }
        out.extend(paras);
    }
    out.join("\n")
}

/// The screen as the board draws it: its prose, wrapped to the width it has.
pub fn lines(text: &str, width: usize) -> Vec<Inked> {
    let mut out: Vec<Inked> = vec![];
    for line in prose(text).split('\n') { out.extend(Inked::plain(line).wrap(width)); }
    if out.is_empty() { out.push(Inked::new()); }
    out
}
