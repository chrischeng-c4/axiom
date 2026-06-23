use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// textwrap module for Mamba (#448, #1261 long-tail wire).
///
/// Provides the public module functions `wrap`, `fill`, `dedent`, `indent`,
/// `shorten` plus the `TextWrapper` class, ported faithfully from CPython
/// 3.12's `Lib/textwrap.py` so that the behaviour, errors and surface
/// fixtures under tests/cpython/.../textwrap match the CPython oracle.
///
/// Module-attr entries are wired through identity-stable callable
/// dispatchers (`unsafe extern "C" fn(args_ptr, nargs)` trampolines) that
/// unpack flat-positional args (with an optional trailing kwargs dict) and
/// call the real Rust impls. The `TextWrapper` class is registered with a
/// method table via `mb_class_register` (same shape as `argparse_mod`); its
/// options live as ordinary instance fields so user code can both read
/// (`wrapper.width`) and mutate (`wrapper.width = 20`) them between wraps.
use std::collections::HashMap;

const WRAPPER_CLASS: &str = "TextWrapper";

// ── Small value helpers ──

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn new_list(items: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(items))
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn is_str(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Str(_)) })
        .unwrap_or(false)
}

fn is_dict(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

fn dict_get(dict: MbValue, key: &str) -> Option<MbValue> {
    dict.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn truthy(val: MbValue) -> bool {
    if val.is_none() {
        return false;
    }
    if let Some(b) = val.as_bool() {
        return b;
    }
    if let Some(i) = val.as_int() {
        return i != 0;
    }
    if let Some(s) = extract_str(val) {
        return !s.is_empty();
    }
    true
}

fn set_field(inst: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(val);
                let prev = fields.write().unwrap().insert(key.to_string(), val);
                if let Some(p) = prev {
                    super::super::rc::release_if_ptr(p);
                }
            }
        }
    }
}

fn get_field(inst: MbValue, key: &str) -> Option<MbValue> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn raise(exc: &str, msg: &str) {
    super::super::exception::mb_raise(new_str(exc), new_str(msg));
}

/// Turn a list/tuple value into a Vec<MbValue>; anything else → empty.
fn seq_items(val: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => return lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => return items.clone(),
                _ => {}
            }
        }
    }
    Vec::new()
}

// ── TextWrapper options ──

#[derive(Clone)]
struct Options {
    width: i64,
    initial_indent: String,
    subsequent_indent: String,
    expand_tabs: bool,
    replace_whitespace: bool,
    fix_sentence_endings: bool,
    break_long_words: bool,
    drop_whitespace: bool,
    break_on_hyphens: bool,
    tabsize: i64,
    max_lines: Option<i64>,
    placeholder: String,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            width: 70,
            initial_indent: String::new(),
            subsequent_indent: String::new(),
            expand_tabs: true,
            replace_whitespace: true,
            fix_sentence_endings: false,
            break_long_words: true,
            drop_whitespace: true,
            break_on_hyphens: true,
            tabsize: 8,
            max_lines: None,
            placeholder: " [...]".to_string(),
        }
    }
}

// CPython: whitespace = ' \t\n\x0b\x0c\r'; replaced with a single space each.
const WHITESPACE: &[char] = &[' ', '\t', '\n', '\x0b', '\x0c', '\r'];

fn is_ws(c: char) -> bool {
    WHITESPACE.contains(&c)
}

/// `[^\d\W]` — a word char that is not a digit (letters + underscore).
fn is_letter(c: char) -> bool {
    if c == '_' {
        return true;
    }
    c.is_alphabetic() && !c.is_numeric()
}

/// `[\w!"'&.,?]` — word char or selected punctuation (the "word_punct" class).
fn is_word_punct(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || matches!(c, '!' | '"' | '\'' | '&' | '.' | ',' | '?')
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

// ── Whitespace munging (expand tabs / replace whitespace) ──

fn munge_whitespace(text: &str, opts: &Options) -> String {
    let mut s = if opts.expand_tabs {
        expand_tabs(text, opts.tabsize as usize)
    } else {
        text.to_string()
    };
    if opts.replace_whitespace {
        s = s.chars().map(|c| if is_ws(c) { ' ' } else { c }).collect();
    }
    s
}

/// Mimic str.expandtabs(tabsize): advance to the next tab stop, resetting the
/// column on newline / carriage return.
fn expand_tabs(text: &str, tabsize: usize) -> String {
    let mut out = String::new();
    let mut col = 0usize;
    for c in text.chars() {
        match c {
            '\t' => {
                if tabsize > 0 {
                    let spaces = tabsize - (col % tabsize);
                    for _ in 0..spaces {
                        out.push(' ');
                    }
                    col += spaces;
                }
            }
            '\n' | '\r' => {
                out.push(c);
                col = 0;
            }
            _ => {
                out.push(c);
                col += 1;
            }
        }
    }
    out
}

// ── Chunk splitting (the wordsep_re port) ──

/// Split `text` into a list of chunks: whitespace runs are their own chunks,
/// and words are split at hyphen / em-dash boundaries when
/// `break_on_hyphens` is true. Faithful port of CPython's wordsep_re /
/// wordsep_simple_re behaviour, verified against the `_split` fixtures.
fn split_chunks(text: &str, break_on_hyphens: bool) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut chunks: Vec<String> = Vec::new();
    let mut i = 0usize;

    while i < n {
        if is_ws(chars[i]) {
            // Whitespace run is one chunk.
            let start = i;
            while i < n && is_ws(chars[i]) {
                i += 1;
            }
            chunks.push(chars[start..i].iter().collect());
            continue;
        }
        // Non-whitespace run [start, end).
        let start = i;
        while i < n && !is_ws(chars[i]) {
            i += 1;
        }
        let end = i;
        if break_on_hyphens {
            split_word(&chars, start, end, &mut chunks);
        } else {
            chunks.push(chars[start..end].iter().collect());
        }
    }

    chunks
}

/// Split a single non-whitespace span [start, end) at hyphen / em-dash break
/// points, pushing each chunk. Mirrors CPython's wordsep_re break rules:
///   - a single hyphen between two letters ends a chunk (hyphen included);
///   - a run of 2+ hyphens (em-dash) preceded by word-punct and followed by a
///     word char is its own chunk, with breaks on either side.
fn split_word(chars: &[char], start: usize, end: usize, chunks: &mut Vec<String>) {
    let mut seg_start = start; // start of the current chunk being built
    let mut k = start;
    while k < end {
        if chars[k] == '-' {
            // Measure the hyphen run at k.
            let mut j = k;
            while j < end && chars[j] == '-' {
                j += 1;
            }
            let run = j - k;
            if run >= 2 {
                // Em-dash candidate: (?<=word_punct) -{2,} (?=\w).
                let before_ok = k > start && is_word_punct(chars[k - 1]);
                let after_ok = j < end && is_word_char(chars[j]);
                if before_ok && after_ok {
                    // Break before the dashes, emit the dash run, continue.
                    if k > seg_start {
                        chunks.push(chars[seg_start..k].iter().collect());
                    }
                    chunks.push(chars[k..j].iter().collect());
                    seg_start = j;
                    k = j;
                    continue;
                }
                // Dashes glued to the word; skip past them.
                k = j;
                continue;
            } else {
                // Single hyphen: break when letter on both sides.
                let prev_letter = k > seg_start && is_letter(chars[k - 1]);
                let next_letter = (k + 1) < end && is_letter(chars[k + 1]);
                if prev_letter && next_letter {
                    // Chunk includes this hyphen.
                    chunks.push(chars[seg_start..=k].iter().collect());
                    seg_start = k + 1;
                }
                k += 1;
                continue;
            }
        }
        k += 1;
    }
    if seg_start < end {
        chunks.push(chars[seg_start..end].iter().collect());
    }
}

// ── Sentence-ending fix ──

fn fix_sentence_endings(chunks: &mut Vec<String>) {
    // After a chunk ending with a sentence-ending punctuation (optionally
    // followed by a closing quote/paren) and whose last letter is lowercase,
    // double a single following space.
    let mut i = 0;
    while i + 1 < chunks.len() {
        if chunks[i + 1] == " " && ends_sentence(&chunks[i]) {
            chunks[i + 1] = "  ".to_string();
            i += 2;
        } else {
            i += 1;
        }
    }
}

/// CPython sentence_end_re: `[a-z][\.\!\?][\"\']?\Z` — lowercase letter,
/// sentence punctuation, optional closing quote, at end of chunk.
fn ends_sentence(chunk: &str) -> bool {
    let cs: Vec<char> = chunk.chars().collect();
    let mut k = cs.len();
    if k == 0 {
        return false;
    }
    // optional trailing " or '
    if cs[k - 1] == '"' || cs[k - 1] == '\'' {
        k -= 1;
    }
    if k == 0 {
        return false;
    }
    let punct = cs[k - 1];
    if punct != '.' && punct != '!' && punct != '?' {
        return false;
    }
    if k < 2 {
        return false;
    }
    let letter = cs[k - 2];
    letter.is_ascii_lowercase()
}

// ── Long-word handling ──

/// Handle a chunk too long to fit on the current line. Mirrors CPython's
/// `_handle_long_word`: pushes a piece of the first chunk onto `cur_line`
/// (mutating `chunks` in place — chunks is stored reversed, last = next word).
fn handle_long_word(
    reversed_chunks: &mut Vec<String>,
    cur_line: &mut Vec<String>,
    cur_len: usize,
    width: i64,
    opts: &Options,
) {
    let space_left: usize = if width < 1 {
        1
    } else {
        (width as usize).saturating_sub(cur_len)
    };

    let last = reversed_chunks.last().cloned().unwrap_or_default();
    let last_chars: Vec<char> = last.chars().collect();

    if opts.break_long_words {
        let mut end = space_left;
        let chunk: &[char] = &last_chars;
        if opts.break_on_hyphens && chunk.len() > space_left {
            // Break after the last hyphen within space_left that is not the
            // first character and not preceded by another hyphen.
            // CPython: hyphen = chunk.rfind('-', 0, space_left)
            //          if hyphen > 0 and any(c != '-' for c in chunk[:hyphen]):
            //              end = hyphen + 1
            let mut hyphen: Option<usize> = None;
            for idx in (0..space_left.min(chunk.len())).rev() {
                if chunk[idx] == '-' {
                    hyphen = Some(idx);
                    break;
                }
            }
            if let Some(h) = hyphen {
                if h > 0 && chunk[..h].iter().any(|&c| c != '-') {
                    end = h + 1;
                }
            }
        }
        let end = end.min(chunk.len());
        let piece: String = chunk[..end].iter().collect();
        cur_line.push(piece);
        let rest: String = chunk[end..].iter().collect();
        // replace the last reversed chunk with the remainder
        let li = reversed_chunks.len() - 1;
        reversed_chunks[li] = rest;
    } else if cur_line.is_empty() {
        // break_long_words is false and the line is empty: take the whole word.
        if let Some(w) = reversed_chunks.pop() {
            cur_line.push(w);
        }
    }
    // else: leave the long word for the next line.
}

// ── Core wrap ──

fn wrap_chunks(mut chunks: Vec<String>, opts: &Options) -> Result<Vec<String>, ()> {
    let mut lines: Vec<String> = Vec::new();
    if opts.width <= 0 {
        raise(
            "ValueError",
            &format!("invalid width {} (must be > 0)", opts.width),
        );
        return Err(());
    }
    if let Some(ml) = opts.max_lines {
        let indent = if ml > 1 {
            &opts.subsequent_indent
        } else {
            &opts.initial_indent
        };
        if (indent.chars().count() + opts.placeholder.trim_start().chars().count())
            > opts.width as usize
        {
            raise("ValueError", "placeholder too large for max width");
            return Err(());
        }
    }

    // chunks is processed from the end (reversed → last = next word).
    chunks.reverse();

    let strip_empty = |s: &str| s.trim().is_empty();

    while !chunks.is_empty() {
        let mut cur_line: Vec<String> = Vec::new();
        let mut cur_len: usize = 0;

        let indent = if !lines.is_empty() {
            opts.subsequent_indent.clone()
        } else {
            opts.initial_indent.clone()
        };

        let width = (opts.width - indent.chars().count() as i64).max(0) as usize;

        // Drop a leading whitespace chunk (unless this is the first line).
        if opts.drop_whitespace
            && chunks.last().map(|c| strip_empty(c)).unwrap_or(false)
            && !lines.is_empty()
        {
            chunks.pop();
        }

        while let Some(last) = chunks.last() {
            let l = last.chars().count();
            if cur_len + l <= width {
                cur_line.push(chunks.pop().unwrap());
                cur_len += l;
            } else {
                break;
            }
        }

        // If the next chunk is too big to fit on *any* line, break it.
        if let Some(last) = chunks.last() {
            if last.chars().count() > width {
                handle_long_word(
                    &mut chunks,
                    &mut cur_line,
                    cur_len,
                    opts.width - indent.chars().count() as i64,
                    opts,
                );
                cur_len = cur_line.iter().map(|s| s.chars().count()).sum();
            }
        }

        // Drop a trailing whitespace chunk on this line.
        if opts.drop_whitespace && cur_line.last().map(|c| strip_empty(c)).unwrap_or(false) {
            cur_len -= cur_line.last().unwrap().chars().count();
            cur_line.pop();
        }

        if !cur_line.is_empty() {
            // CPython condition (note operator precedence: && binds tighter
            // than ||, and the trailing `&& cur_len <= width` applies to the
            // *whole* second/third || branch group):
            //   max_lines is None
            //   or len(lines)+1 < max_lines
            //   or ((not chunks) or (drop_whitespace and len==1 and !strip))
            //       and cur_len <= width
            let simple = match opts.max_lines {
                None => true,
                Some(ml) => {
                    if (lines.len() as i64) + 1 < ml {
                        true
                    } else {
                        let no_more = chunks.is_empty()
                            || (opts.drop_whitespace
                                && chunks.len() == 1
                                && chunks[0].trim().is_empty());
                        no_more && cur_len <= width
                    }
                }
            };

            if simple {
                lines.push(format!("{}{}", indent, cur_line.concat()));
                continue;
            }

            // max_lines reached: append a placeholder, backtracking to fit it.
            let mut placed = false;
            while !cur_line.is_empty() {
                if !cur_line.last().unwrap().trim().is_empty()
                    && cur_len + opts.placeholder.chars().count() <= width
                {
                    cur_line.push(opts.placeholder.clone());
                    lines.push(format!("{}{}", indent, cur_line.concat()));
                    placed = true;
                    break;
                }
                let removed = cur_line.pop().unwrap();
                cur_len -= removed.chars().count();
            }
            if !placed {
                if !lines.is_empty() {
                    let prev_line = lines.last().unwrap().trim_end().to_string();
                    if prev_line.chars().count() + opts.placeholder.chars().count()
                        <= opts.width as usize
                    {
                        let li = lines.len() - 1;
                        lines[li] = format!("{}{}", prev_line, opts.placeholder);
                        return Ok(lines);
                    }
                }
                lines.push(format!("{}{}", indent, opts.placeholder.trim_start()));
            }
            return Ok(lines);
        }
    }

    Ok(lines)
}

/// Run the full pipeline: munge → split → fix-sentence → wrap.
fn do_wrap(text: &str, opts: &Options) -> Result<Vec<String>, ()> {
    let prepared = munge_whitespace(text, opts);
    let mut chunks = split_chunks(&prepared, opts.break_on_hyphens);
    if opts.fix_sentence_endings {
        fix_sentence_endings(&mut chunks);
    }
    wrap_chunks(chunks, opts)
}

// ── dedent / indent / shorten ──

fn dedent_impl(text: &str) -> String {
    // CPython textwrap.dedent: longest common leading-whitespace prefix among
    // lines that contain non-whitespace; whitespace-only lines normalised to
    // empty and ignored for prefix computation.
    let lines: Vec<&str> = text.split('\n').collect();
    let mut margin: Option<String> = None;
    let mut normalised: Vec<String> = Vec::with_capacity(lines.len());

    for line in &lines {
        let stripped = line.trim_start_matches(|c: char| c == ' ' || c == '\t');
        if stripped.is_empty() {
            // Whitespace-only line: normalise to empty, ignore for prefix.
            normalised.push(String::new());
            continue;
        }
        let indent_len = line.len() - stripped.len();
        let indent = &line[..indent_len];
        normalised.push(line.to_string());
        margin = Some(match margin {
            None => indent.to_string(),
            Some(m) => common_prefix(&m, indent),
        });
    }

    let margin = margin.unwrap_or_default();
    let out: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(idx, line)| {
            let stripped = line.trim_start_matches(|c: char| c == ' ' || c == '\t');
            if stripped.is_empty() {
                // whitespace-only → blank
                let _ = idx;
                String::new()
            } else if line.starts_with(&margin) {
                line[margin.len()..].to_string()
            } else {
                line.to_string()
            }
        })
        .collect();
    out.join("\n")
}

fn common_prefix(a: &str, b: &str) -> String {
    let mut out = String::new();
    for (ca, cb) in a.chars().zip(b.chars()) {
        if ca == cb {
            out.push(ca);
        } else {
            break;
        }
    }
    out
}

fn indent_impl(text: &str, prefix: &str, predicate: Option<MbValue>) -> MbValue {
    // CPython indent: split keeping line endings; prefix every line for which
    // the predicate is true (default: line.strip() truthy).
    let mut out = String::new();
    for line in split_keepends(text) {
        let stripped = line.trim();
        let do_prefix = match predicate {
            Some(p) => {
                let r = super::super::class::mb_call1_val(p, new_str(&line));
                truthy(r)
            }
            None => !stripped.is_empty(),
        };
        if do_prefix {
            out.push_str(prefix);
        }
        out.push_str(&line);
    }
    new_str(&out)
}

/// Split keeping line endings (Python str.splitlines(keepends=True)).
fn split_keepends(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        cur.push(c);
        if c == '\n' {
            out.push(std::mem::take(&mut cur));
        } else if c == '\r' {
            if i + 1 < chars.len() && chars[i + 1] == '\n' {
                cur.push('\n');
                i += 1;
            }
            out.push(std::mem::take(&mut cur));
        }
        i += 1;
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn shorten_impl(text: &str, opts: &Options) -> Result<MbValue, ()> {
    // CPython shorten: collapse whitespace, then wrap with max_lines=1.
    let collapsed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut o = opts.clone();
    o.max_lines = Some(1);
    o.initial_indent = String::new();
    o.subsequent_indent = String::new();
    let lines = do_wrap(&collapsed, &o)?;
    if lines.is_empty() {
        Ok(new_str(""))
    } else {
        Ok(new_str(&lines[0]))
    }
}

// ── Options from kwargs ──

fn opts_from_kwargs(base: Options, kwargs: MbValue) -> Options {
    let mut o = base;
    if !is_dict(kwargs) {
        return o;
    }
    if let Some(v) = dict_get(kwargs, "width") {
        if let Some(i) = v.as_int() {
            o.width = i;
        }
    }
    if let Some(v) = dict_get(kwargs, "initial_indent") {
        if let Some(s) = extract_str(v) {
            o.initial_indent = s;
        }
    }
    if let Some(v) = dict_get(kwargs, "subsequent_indent") {
        if let Some(s) = extract_str(v) {
            o.subsequent_indent = s;
        }
    }
    if let Some(v) = dict_get(kwargs, "expand_tabs") {
        o.expand_tabs = truthy(v);
    }
    if let Some(v) = dict_get(kwargs, "replace_whitespace") {
        o.replace_whitespace = truthy(v);
    }
    if let Some(v) = dict_get(kwargs, "fix_sentence_endings") {
        o.fix_sentence_endings = truthy(v);
    }
    if let Some(v) = dict_get(kwargs, "break_long_words") {
        o.break_long_words = truthy(v);
    }
    if let Some(v) = dict_get(kwargs, "drop_whitespace") {
        o.drop_whitespace = truthy(v);
    }
    if let Some(v) = dict_get(kwargs, "break_on_hyphens") {
        o.break_on_hyphens = truthy(v);
    }
    if let Some(v) = dict_get(kwargs, "tabsize") {
        if let Some(i) = v.as_int() {
            o.tabsize = i;
        }
    }
    if let Some(v) = dict_get(kwargs, "max_lines") {
        if v.is_none() {
            o.max_lines = None;
        } else if let Some(i) = v.as_int() {
            o.max_lines = Some(i);
        }
    }
    if let Some(v) = dict_get(kwargs, "placeholder") {
        if let Some(s) = extract_str(v) {
            o.placeholder = s;
        }
    }
    o
}

fn opts_from_instance(inst: MbValue) -> Options {
    let mut o = Options::default();
    if let Some(v) = get_field(inst, "width").and_then(|v| v.as_int()) {
        o.width = v;
    }
    if let Some(s) = get_field(inst, "initial_indent").and_then(extract_str) {
        o.initial_indent = s;
    }
    if let Some(s) = get_field(inst, "subsequent_indent").and_then(extract_str) {
        o.subsequent_indent = s;
    }
    if let Some(v) = get_field(inst, "expand_tabs") {
        o.expand_tabs = truthy(v);
    }
    if let Some(v) = get_field(inst, "replace_whitespace") {
        o.replace_whitespace = truthy(v);
    }
    if let Some(v) = get_field(inst, "fix_sentence_endings") {
        o.fix_sentence_endings = truthy(v);
    }
    if let Some(v) = get_field(inst, "break_long_words") {
        o.break_long_words = truthy(v);
    }
    if let Some(v) = get_field(inst, "drop_whitespace") {
        o.drop_whitespace = truthy(v);
    }
    if let Some(v) = get_field(inst, "break_on_hyphens") {
        o.break_on_hyphens = truthy(v);
    }
    if let Some(v) = get_field(inst, "tabsize").and_then(|v| v.as_int()) {
        o.tabsize = v;
    }
    if let Some(v) = get_field(inst, "max_lines") {
        if v.is_none() {
            o.max_lines = None;
        } else if let Some(i) = v.as_int() {
            o.max_lines = Some(i);
        }
    }
    if let Some(s) = get_field(inst, "placeholder").and_then(extract_str) {
        o.placeholder = s;
    }
    o
}

// ── Module-function dispatchers (native; trailing kwargs dict honoured) ──

/// Safely view the native arg slice. When called with zero arguments the
/// runtime may pass a null/dangling `args_ptr`; `slice::from_raw_parts`
/// requires a non-null aligned pointer even for a zero-length slice, so guard
/// against that and return an empty slice instead of panicking.
unsafe fn arg_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    }
}

/// Pull (positional[..], kwargs_dict) out of a native arg slice.
fn split_args(a: &[MbValue]) -> (Vec<MbValue>, MbValue) {
    if let Some(last) = a.last() {
        if is_dict(*last) {
            return (a[..a.len() - 1].to_vec(), *last);
        }
    }
    (a.to_vec(), MbValue::none())
}

/// textwrap.wrap(text, width=70, **kwargs) -> list[str]
unsafe extern "C" fn dispatch_wrap(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let (pos, kwargs) = split_args(a);
    let text = match pos.first().copied().and_then(extract_str) {
        Some(t) => t,
        None => return new_list(vec![]),
    };
    let mut base = Options::default();
    if let Some(w) = pos.get(1).copied().and_then(|v| v.as_int()) {
        base.width = w;
    }
    let opts = opts_from_kwargs(base, kwargs);
    match do_wrap(&text, &opts) {
        Ok(lines) => new_list(lines.iter().map(|l| new_str(l)).collect()),
        Err(()) => MbValue::none(),
    }
}

/// textwrap.fill(text, width=70, **kwargs) -> str
unsafe extern "C" fn dispatch_fill(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let (pos, kwargs) = split_args(a);
    let text = match pos.first().copied().and_then(extract_str) {
        Some(t) => t,
        None => return new_str(""),
    };
    let mut base = Options::default();
    if let Some(w) = pos.get(1).copied().and_then(|v| v.as_int()) {
        base.width = w;
    }
    let opts = opts_from_kwargs(base, kwargs);
    match do_wrap(&text, &opts) {
        Ok(lines) => new_str(&lines.join("\n")),
        Err(()) => MbValue::none(),
    }
}

/// textwrap.dedent(text) -> str  (TypeError on non-str).
unsafe extern "C" fn dispatch_dedent(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let (pos, _kw) = split_args(a);
    let arg = pos.first().copied().unwrap_or_else(MbValue::none);
    if !is_str(arg) {
        raise("TypeError", "expected str object");
        return MbValue::none();
    }
    let s = extract_str(arg).unwrap_or_default();
    new_str(&dedent_impl(&s))
}

/// textwrap.indent(text, prefix, predicate=None) -> str.
unsafe extern "C" fn dispatch_indent(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let (pos, kwargs) = split_args(a);
    let text_v = pos.first().copied().unwrap_or_else(MbValue::none);
    if !is_str(text_v) {
        // CPython does text.splitlines(True) → AttributeError on non-str.
        raise(
            "AttributeError",
            "'int' object has no attribute 'splitlines'",
        );
        return MbValue::none();
    }
    let text = extract_str(text_v).unwrap_or_default();
    let prefix = pos
        .get(1)
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    let predicate = pos
        .get(2)
        .copied()
        .filter(|v| !v.is_none())
        .or_else(|| dict_get(kwargs, "predicate").filter(|v| !v.is_none()));
    indent_impl(&text, &prefix, predicate)
}

/// textwrap.shorten(text, width, **kwargs) -> str.
unsafe extern "C" fn dispatch_shorten(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let (pos, kwargs) = split_args(a);
    let text = match pos.first().copied().and_then(extract_str) {
        Some(t) => t,
        None => return new_str(""),
    };
    let mut base = Options::default();
    if let Some(w) = pos.get(1).copied().and_then(|v| v.as_int()) {
        base.width = w;
    }
    let opts = opts_from_kwargs(base, kwargs);
    match shorten_impl(&text, &opts) {
        Ok(v) => v,
        Err(()) => MbValue::none(),
    }
}

// ── TextWrapper constructor + methods ──

/// TextWrapper(width=70, **kwargs) -> instance with option fields set.
unsafe extern "C" fn dispatch_text_wrapper(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let (pos, kwargs) = split_args(a);
    let inst = MbValue::from_ptr(MbObject::new_instance(WRAPPER_CLASS.to_string()));

    // Defaults.
    let d = Options::default();
    set_field(inst, "width", MbValue::from_int(d.width));
    set_field(inst, "initial_indent", new_str(&d.initial_indent));
    set_field(inst, "subsequent_indent", new_str(&d.subsequent_indent));
    set_field(inst, "expand_tabs", MbValue::from_bool(d.expand_tabs));
    set_field(
        inst,
        "replace_whitespace",
        MbValue::from_bool(d.replace_whitespace),
    );
    set_field(
        inst,
        "fix_sentence_endings",
        MbValue::from_bool(d.fix_sentence_endings),
    );
    set_field(
        inst,
        "break_long_words",
        MbValue::from_bool(d.break_long_words),
    );
    set_field(
        inst,
        "drop_whitespace",
        MbValue::from_bool(d.drop_whitespace),
    );
    set_field(
        inst,
        "break_on_hyphens",
        MbValue::from_bool(d.break_on_hyphens),
    );
    set_field(inst, "tabsize", MbValue::from_int(d.tabsize));
    set_field(inst, "max_lines", MbValue::none());
    set_field(inst, "placeholder", new_str(&d.placeholder));

    // First positional is width.
    if let Some(w) = pos.first().copied() {
        set_field(inst, "width", w);
    }

    // Apply kwargs onto the matching fields.
    for key in [
        "width",
        "initial_indent",
        "subsequent_indent",
        "expand_tabs",
        "replace_whitespace",
        "fix_sentence_endings",
        "break_long_words",
        "drop_whitespace",
        "break_on_hyphens",
        "tabsize",
        "max_lines",
        "placeholder",
    ] {
        if let Some(v) = dict_get(kwargs, key) {
            set_field(inst, key, v);
        }
    }

    inst
}

/// wrapper.wrap(text) -> list[str]
unsafe extern "C" fn method_wrap(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let text = items
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    let opts = opts_from_instance(self_v);
    match do_wrap(&text, &opts) {
        Ok(lines) => new_list(lines.iter().map(|l| new_str(l)).collect()),
        Err(()) => MbValue::none(),
    }
}

/// wrapper.fill(text) -> str
unsafe extern "C" fn method_fill(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let text = items
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    let opts = opts_from_instance(self_v);
    match do_wrap(&text, &opts) {
        Ok(lines) => new_str(&lines.join("\n")),
        Err(()) => MbValue::none(),
    }
}

/// wrapper._split(text) -> list[str]  (test-facing helper).
unsafe extern "C" fn method_split(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let text = items
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    let opts = opts_from_instance(self_v);
    let prepared = munge_whitespace(&text, &opts);
    let chunks = split_chunks(&prepared, opts.break_on_hyphens);
    new_list(chunks.iter().map(|c| new_str(c)).collect())
}

/// wrapper._munge_whitespace(text) -> str.
unsafe extern "C" fn method_munge(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let text = items
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    let opts = opts_from_instance(self_v);
    new_str(&munge_whitespace(&text, &opts))
}

// ── Registration ──

pub fn register() {
    register_classes();

    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("wrap", dispatch_wrap as *const () as usize),
        ("fill", dispatch_fill as *const () as usize),
        ("dedent", dispatch_dedent as *const () as usize),
        ("indent", dispatch_indent as *const () as usize),
        ("shorten", dispatch_shorten as *const () as usize),
        ("TextWrapper", dispatch_text_wrapper as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    attrs.insert(
        "__all__".into(),
        new_list(
            ["TextWrapper", "wrap", "fill", "dedent", "indent", "shorten"]
                .iter()
                .map(|s| new_str(s))
                .collect(),
        ),
    );

    // Bridge the TextWrapper constructor func -> its class name so accessing a
    // class attribute method (`textwrap.TextWrapper.fill`) flows through
    // mb_getattr's func->native-class method bridge (which looks the func addr
    // up in NATIVE_TYPE_NAMES, then lookup_method in the table that
    // mb_class_register populated). Without this the methods are registered but
    // `hasattr(textwrap.TextWrapper, "fill")` / `callable(...)` is False.
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(
            dispatch_text_wrapper as *const () as usize as u64,
            WRAPPER_CLASS.to_string(),
        );
    });

    super::register_module("textwrap", attrs);
}

fn register_classes() {
    let mut map: HashMap<String, MbValue> = HashMap::new();
    let methods: &[(&str, usize)] = &[
        ("wrap", method_wrap as usize),
        ("fill", method_fill as usize),
        ("_split", method_split as usize),
        ("_munge_whitespace", method_munge as usize),
    ];
    for (name, addr) in methods {
        map.insert(name.to_string(), MbValue::from_func(*addr));
        super::super::module::register_variadic_func(*addr as u64);
    }
    super::super::class::mb_class_register(WRAPPER_CLASS, vec![], map);
}

// Legacy public API kept for any symbols.rs JIT entries that referenced the
// old positional shims.

pub fn mb_textwrap_wrap(text: MbValue, width: MbValue) -> MbValue {
    let s = extract_str(text).unwrap_or_default();
    let mut o = Options::default();
    if let Some(w) = width.as_int() {
        o.width = w;
    }
    match do_wrap(&s, &o) {
        Ok(lines) => new_list(lines.iter().map(|l| new_str(l)).collect()),
        Err(()) => MbValue::none(),
    }
}

pub fn mb_textwrap_fill(text: MbValue, width: MbValue) -> MbValue {
    let s = extract_str(text).unwrap_or_default();
    let mut o = Options::default();
    if let Some(w) = width.as_int() {
        o.width = w;
    }
    match do_wrap(&s, &o) {
        Ok(lines) => new_str(&lines.join("\n")),
        Err(()) => MbValue::none(),
    }
}

pub fn mb_textwrap_dedent(text: MbValue) -> MbValue {
    let s = extract_str(text).unwrap_or_default();
    new_str(&dedent_impl(&s))
}

pub fn mb_textwrap_indent(text: MbValue, prefix: MbValue) -> MbValue {
    let s = extract_str(text).unwrap_or_default();
    let p = extract_str(prefix).unwrap_or_default();
    indent_impl(&s, &p, None)
}

pub fn mb_textwrap_shorten(text: MbValue, width: MbValue) -> MbValue {
    let s = extract_str(text).unwrap_or_default();
    let mut o = Options::default();
    if let Some(w) = width.as_int() {
        o.width = w;
    }
    match shorten_impl(&s, &o) {
        Ok(v) => v,
        Err(()) => MbValue::none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedent_common_prefix() {
        assert_eq!(dedent_impl("  hello\n  world"), "hello\nworld");
        assert_eq!(
            dedent_impl("   line1\n   line2\n     line3\n"),
            "line1\nline2\n  line3\n"
        );
    }

    #[test]
    fn test_dedent_ws_only_line() {
        assert_eq!(
            dedent_impl("  Hello there.\n  \n  How are ya?\n  Oh good.\n"),
            "Hello there.\n\nHow are ya?\nOh good.\n"
        );
    }

    #[test]
    fn test_split_simple() {
        let c = split_chunks("Hello there -- you goof-ball, use the -b option!", true);
        assert_eq!(
            c,
            vec![
                "Hello", " ", "there", " ", "--", " ", "you", " ", "goof-", "ball,", " ", "use",
                " ", "the", " ", "-b", " ", "option!"
            ]
        );
    }

    #[test]
    fn test_split_funky_hyphens() {
        assert_eq!(
            split_chunks("what the--hey!", true),
            vec!["what", " ", "the", "--", "hey!"]
        );
        assert_eq!(split_chunks("what the--", true), vec!["what", " ", "the--"]);
        assert_eq!(split_chunks("--option-opt", true), vec!["--option-", "opt"]);
    }

    #[test]
    fn test_wrap_break_long() {
        let mut o = Options::default();
        o.width = 10;
        let r = do_wrap("ab longer_word cd", &o).unwrap();
        assert!(r.iter().all(|l| l.chars().count() <= 10));
    }
}
