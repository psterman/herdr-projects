//! The words on a program's screen, for a program whose saved conversation the board cannot read. What a terminal
//! program draws is told from what it says by its shape alone: a line that is mostly drawing (a rule, a box's edge, a
//! logo made of dots or blocks, a bar) holds few letters among its marks, and is left out; a box's edges are taken off.
//! The rows a program draws for itself are left out too: the bar of fields that shows its model and its tokens, and
//! the box it types in, which stands at the bottom of the screen.
const EDGE: &[char] = &['│', '┃', '|', '╭', '╮', '╰', '╯', '┌', '┐', '└', '┘', '─', '━', '═'];
/// The sign a program shows where its words are typed in.
const PROMPT: &[char] = &['❯', '›', '>'];
/// The sides of a box: a line between them is written inside it.
const SIDE: &[char] = &['│', '┃', '|'];
fn edge_trim(s: &str) -> &str { s.trim_matches(|c: char| c.is_whitespace() || EDGE.contains(&c)) }
fn drawn(line: &str) -> bool {
    let marks: Vec<char> = line.chars().filter(|c| !c.is_whitespace()).collect();
    let words = marks.iter().filter(|c| c.is_alphanumeric()).count();
    !marks.is_empty() && words * 2 < marks.len()
}
/// A row of short fields, separated by bars: the status a program shows of itself. The last field may be long: a
/// title hangs at the end of the bar.
fn fields(line: &str) -> bool {
    let parts: Vec<&str> = line.split('│').map(|p| p.trim()).collect();
    parts.len() >= 3 && parts[..parts.len() - 1].iter().all(|p| !p.is_empty() && p.chars().count() <= 24) && !parts[parts.len() - 1].is_empty()
}
/// A line that is only a rule or the edge of a box.
fn rule(raw: &str) -> bool {
    let t = raw.trim();
    !t.is_empty() && t.chars().all(|c| EDGE.contains(&c) || c.is_whitespace())
}
/// The line a program types in: the last that stands at a prompt, at the bottom of the screen, and not written inside
/// a box of its own (a menu of choices is).
fn input_line(lines: &[&str]) -> Option<usize> {
    let last = lines.iter().rposition(|l| { let t = l.trim_start(); !t.is_empty() && PROMPT.contains(&t.chars().next().unwrap_or(' ')) })?;
    let t = lines[last].trim_start();
    let framed = t.chars().next().map(|c| SIDE.contains(&c)).unwrap_or(false) || t.trim_end().chars().last().map(|c| SIDE.contains(&c)).unwrap_or(false);
    if framed { return None; }
    if lines[last + 1..].iter().any(|l| { let t = l.trim(); !t.is_empty() && !rule(l) && !drawn(edge_trim(l)) }) { return None; }
    Some(last)
}
pub fn screen_text(screen: &str) -> String {
    let lines: Vec<&str> = screen.split('\n').collect();
    let input = input_line(&lines);
    let mut out: Vec<String> = vec![];
    for (n, raw) in lines.iter().enumerate() {
        let line = edge_trim(raw);
        if !raw.trim().is_empty() && (line.is_empty() || drawn(line)) { continue; }
        if line.is_empty() && out.last().map(|l| l.is_empty()).unwrap_or(true) { continue; } // one empty line between blocks
        if Some(n) == input { continue; } // the box it types in
        if !line.is_empty() && fields(line) { continue; } // the bar of status it draws for itself
        if line.is_empty() { out.push(String::new()); continue; }
        let mut s = raw.trim_start();
        if let Some(r) = s.strip_prefix(['│', '┃', '|']) { s = r.strip_prefix(' ').unwrap_or(r); } else { s = raw; }
        let s = s.trim_end();
        let s = s.strip_suffix(['│', '┃', '|']).map(|x| x.trim_end()).unwrap_or(s);
        out.push(s.to_string());
    }
    while out.last().map(|l| l.is_empty()).unwrap_or(false) { out.pop(); }
    out.join("\n")
}
