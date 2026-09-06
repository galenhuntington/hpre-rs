const TAB_WIDTH: usize = 3;
const IMPORT_MARKER: &str = "--+";
const ELSE_WORD: &str = "True";

fn is_space(c: char) -> bool { c == ' ' }
fn isnt_space(c: char) -> bool { c != ' ' }

/// Split `s` at the first char matching `pat`; the full string if none match.
fn split_at_find(s: &str, pat: impl FnMut(char) -> bool) -> (&str, &str) {
    s.split_at(s.find(pat).unwrap_or(s.len()))
}

fn untab(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut col: usize = 0;
    for c in line.chars() {
        if c == '\t' {
            let spaces = TAB_WIDTH - (col % TAB_WIDTH);
            out.extend(std::iter::repeat_n(' ', spaces));
            col += spaces;
        } else {
            out.push(c);
            col += 1;
        }
    }
    out
}

fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || c == '\'' || c == '_'
}

fn empty_guard(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        out.push(c);
        match c {
            // Char literal: pass-through of one char handles `'"'`,
            // which appears to be the only case of concern.
            '\'' => if let Some(c1) = chars.next() { out.push(c1) }
            // Limitation: No multi-line quotes, stops at end of line.
            '"' =>
                while let Some(c) = chars.next() {
                    out.push(c);
                    match c {
                        '"'  => break,
                        '\\' => if let Some(c1) = chars.next() { out.push(c1) }
                        _    => {}
                    }
                }
            // XXX if let guards landed in Rust 1.94
            ' ' => if let Some(after_bar) = chars.as_str().strip_prefix("| ") {
                out.push('|');
                let (spaces, tail) = split_at_find(after_bar, isnt_space);
                if matches!(split_at_find(tail, is_space).0, "=" | "->" | "→") {
                    let ew_len = ELSE_WORD.len();
                    if spaces.len() < ew_len {
                        out.push_str(ELSE_WORD);
                    } else {
                        out.push(' ');
                        out.push_str(ELSE_WORD);
                        out.push_str(&spaces[ew_len..]);
                    }
                } else {
                    out.push(' ');
                    out.push_str(spaces);
                }
                chars = tail.chars();
            }
            _ => {}
        }
    }
    out
}

type DittoHist = Vec<(usize, String)>;

fn hist_to_indent(ind: usize, hist: &mut DittoHist) -> Option<String> {
    let pos = hist.iter().position(|(i, _)| *i >= ind)?;
    hist.drain(pos..).next().filter(|(i, _)| *i == ind).map(|(_, s)| s)
}

fn expand_ditto(line: &mut String, val: &str) {
    let (pre, rest) = split_at_find(line, isnt_space);
    let tail = split_at_find(split_at_find(rest, is_space).1, isnt_space).1;
    let cut = line.len() - tail.len();
    // Column pragma takes _char_ count.
    let col = line[..cut].chars().count() + 1;
    line.replace_range(pre.len()..cut, &format!("{}{{-#COLUMN {}#-}}", val, col))
}

fn find_lets(line: &str, hist: &mut DittoHist) {
    let mut rest = line;
    let mut col = 0;
    let mut let_last = false;
    while !rest.is_empty() {
        let (sep, after_sep) = split_at_find(rest, is_name_char);
        let name_col = col + sep.chars().count();
        let (name, tail) = split_at_find(after_sep, |c| !is_name_char(c));
        if let_last && sep.chars().all(is_space) && !name.is_empty() {
            hist.push((name_col, name.to_owned()));
        }
        let_last = name == "let";
        col = name_col + name.chars().count();
        rest = tail;
    }
}

fn ditto_marks(lines: &mut [String]) -> Result<(), String> {
    let mut hist: DittoHist = vec![];
    for line in lines.iter_mut() {
        let (leading, rest) = split_at_find(line, isnt_space);
        if rest.is_empty() || rest.starts_with("--")
            || rest.starts_with("{-") || rest.starts_with("-}")
        {
            continue;
        }
        let indent = leading.len();
        let cur = hist_to_indent(indent, &mut hist);
        if matches!(split_at_find(rest, is_space).0, "''" | "”" | "〃") {
            let Some(cur) = cur else {
                return Err(format!("Orphaned ditto mark:  {}", line));
            };
            hist.push((indent, cur.clone()));
            find_lets(line, &mut hist);
            expand_ditto(line, &cur);
        } else {
            if rest.starts_with(char::is_lowercase) {
                let name = split_at_find(rest, |c| !is_name_char(c)).0;
                hist.push((indent, name.to_string()));
            }
            find_lets(line, &mut hist);
        }
    }
    Ok(())
}

/// Strip trailing comments and space
/// Limitation: Not aware of quoted `--`.
fn strip_end_fluff(s: &str) -> &str {
    if s.starts_with("--") {
        ""
    } else {
        s.find(" -- ").map_or(s, |i| &s[..i]).trim_end()
    }
}

fn is_only_comment(s: &str) -> bool {
    let s = s.trim_start();
    s.is_empty() || s.starts_with("-- ")
}

fn commas_r(lines: &mut [String]) {
    let mut it = lines.iter_mut();
    while let Some(line) = it.next() {
        let dec = strip_end_fluff(line);
        if dec.ends_with(',')
            && it.as_slice().iter().find(|l| !is_only_comment(l))
                .is_some_and(|s| s.trim_start().starts_with([')', ']', '}']))
        {
            line.truncate(dec.len() - 1);
        }
    }
}

fn transform_l(lines: &mut [String], c: char, guard: impl Fn(&str) -> bool) {
    let mut it = lines.iter_mut();
    while let Some(line) = it.next_back() {
        if let Some(pos) = line.find(isnt_space)
            && line[pos..].starts_with(c)
            && it.as_slice().iter().rfind(|l| !is_only_comment(l))
                .is_some_and(|s| guard(strip_end_fluff(s)))
        {
            line.replace_range(pos..=pos, " ");
        }
    }
}

fn commas_l(lines: &mut [String]) {
    transform_l(lines, ',', |l| l.ends_with(['{', '(', '[']))
}

fn data_bars_l(lines: &mut [String]) {
    transform_l(lines, '|', |l| l.starts_with("data ") && l.ends_with('='))
}

fn parse_import_name(blk: &str) -> (&str, &str) {
    let trimmed = blk.trim_ascii_start();
    let split = |s| split_at_find(s, |c| c == ',' || c.is_whitespace());
    // PackageImports
    if let Some(after_open) = trimmed.strip_prefix('"')
        && let Some(end_quote) = after_open.find('"')
    {
        let after_quote = after_open[end_quote+1..].trim_ascii_start();
        let rest = split(after_quote).1;
        trimmed.split_at(trimmed.len() - rest.len())
    } else {
        split(trimmed)
    }
}

fn render_import(acc: &str, name: &str) -> String {
    let trimmed = acc.trim_start();
    let qualified = if trimmed.starts_with("as") { "qualified " } else { "" };
    let acc = acc.strip_suffix("(..)").unwrap_or(acc);
    format!("import {}{}{}", qualified, name, acc)
}

/// Split on top-level commas, render as imports joined with `;`.
/// Trailing comma is dropped.
fn do_import(blk: &str) -> String {
    let (name, nx) = parse_import_name(blk);
    let mut list = vec![];
    let mut segment = String::new();
    let mut depth: u32 = 0;
    macro_rules! flush {() => {
        list.push(render_import(&segment, name));
        segment.clear();
    }}
    for c in nx.chars() {
        match c {
            '(' => { depth += 1; segment.push('('); }
            ')' if depth > 0 => { depth -= 1; segment.push(')'); }
            ',' if depth == 0 => { flush!(); }
            c => segment.push(c),
        }
    }
    if list.is_empty() || !segment.chars().all(char::is_whitespace) {
        flush!();
    }
    list.join(";") + &segment
}

// After this "lines" may not be individual lines but blocks.
// However, line numbers should still be preserved.
fn imports(lines: &mut Vec<String>) -> Result<(), String> {
    let Some(pos) = lines.iter().position(|l| l == IMPORT_MARKER) else {
        return Ok(())
    };
    let mut it = lines.drain(pos..).peekable();
    let mut blocks = Vec::with_capacity(it.len());
    while let Some(line) = it.next() {
        blocks.push(
            if line == IMPORT_MARKER {
                String::new()
            } else if let Some(after) = line.strip_prefix("import ") {
                let after = after.trim_start();
                if after.starts_with("qualified") {
                    return Err("'qualified' in multiplex import.".to_string());
                }
                let mut blk = strip_end_fluff(after).to_owned();
                while let Some(l) = it.next_if(
                    |l| l.is_empty() || l.starts_with(' '))
                {
                    blk.push('\n');
                    blk.push_str(strip_end_fluff(&l));
                }
                do_import(&blk)
            } else {
                line
            }
        );
    }
    drop(it);
    lines.extend(blocks);
    Ok(())
}

pub fn process(input: &str) -> Result<String, String> {
    let mut lines: Vec<String> =
        input.lines().map(|l| empty_guard(&untab(l))).collect();
    ditto_marks(&mut lines)?;
    commas_r(&mut lines);
    commas_l(&mut lines);
    data_bars_l(&mut lines);
    imports(&mut lines)?;
    Ok(lines.join("\n") + "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_end_fluff() {
        assert_eq!(strip_end_fluff(""), "");
        assert_eq!(strip_end_fluff("foo -- bar"), "foo");
        assert_eq!(strip_end_fluff("foo    -- bar"), "foo");
        assert_eq!(strip_end_fluff("-- bar"), "");
        assert_eq!(strip_end_fluff("   -- bar"), "");
        // Not a comment unless preceded by a space.
        assert_eq!(strip_end_fluff("(+--) x"), "(+--) x");
        assert_eq!(strip_end_fluff("3 +-- 5,"), "3 +-- 5,");
        assert_eq!(strip_end_fluff("+ -- x"), "+");
    }

    #[test]
    fn tabbing() {
        if TAB_WIDTH < 2 { unreachable!("Tab width must be at least 2.") }
        let sp = " ".repeat(TAB_WIDTH);
        let sp1 = " ".repeat(TAB_WIDTH - 1);
        assert_eq!(untab("\t"), sp);
        assert_eq!(untab("\ta"), format!("{sp}a"));
        assert_eq!(untab("a\t"), format!("a{sp1}"));
        assert_eq!(untab("a\tb\tc"), format!("a{sp1}b{sp1}c"));
        if TAB_WIDTH == 3 {
            assert_eq!(untab("ab\t"), "ab ");
            assert_eq!(untab("abc\t"), "abc   ");
            assert_eq!(untab("abc\td"), "abc   d");
        }
    }

    #[test]
    fn test_empty_guard() {
        // Empty guard: `| = ...` fits `True` as best as possible.
        assert_eq!(empty_guard("x | = 1"), "x |True= 1");
        assert_eq!(empty_guard("x | -> 1"), "x |True-> 1");
        assert_eq!(empty_guard("x | → 1"), "x |True→ 1");
        assert_eq!(empty_guard("x |    = 1"), "x |True= 1");
        assert_eq!(empty_guard("x |     = 1"), "x | True= 1");
        assert_eq!(empty_guard("x |      = 1"), "x | True = 1");
        assert_eq!(empty_guard("x |       = 1"), "x | True  = 1");
        // Non-empty guard (condition before `=`) is left alone.
        assert_eq!(empty_guard("x | y = 1"), "x | y = 1");
        // `|` inside a literal not expanded.
        assert_eq!(empty_guard("s = \" | = \""), "s = \" | = \"");
        assert_eq!(empty_guard("s = \"\\\" | = \""), "s = \"\\\" | = \"");
        assert_eq!(empty_guard("c = '|'"), "c = '|'");
    }
}

