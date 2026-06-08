use logos::Logos;

/// Map a Unicode character name (per Python's `\N{name}` syntax) to a char.
/// Contains a small but representative set of common Unicode names.
fn unicode_name_to_char(name: &str) -> Option<char> {
    match name {
        "NULL" => Some('\0'),
        "SNOWMAN" => Some('\u{2603}'),
        "COPYRIGHT SIGN" => Some('\u{00A9}'),
        "REGISTERED SIGN" => Some('\u{00AE}'),
        "TRADE MARK SIGN" => Some('\u{2122}'),
        "LATIN SMALL LETTER A" => Some('a'),
        "LATIN SMALL LETTER B" => Some('b'),
        "LATIN SMALL LETTER C" => Some('c'),
        "LATIN SMALL LETTER Z" => Some('z'),
        "LATIN CAPITAL LETTER A" => Some('A'),
        "LATIN CAPITAL LETTER Z" => Some('Z'),
        "DIGIT ZERO" => Some('0'),
        "DIGIT ONE" => Some('1'),
        "DIGIT NINE" => Some('9'),
        "SPACE" => Some(' '),
        "EXCLAMATION MARK" => Some('!'),
        "QUESTION MARK" => Some('?'),
        "FULL STOP" => Some('.'),
        "COMMA" => Some(','),
        "COLON" => Some(':'),
        "SEMICOLON" => Some(';'),
        "SOLIDUS" | "SLASH" => Some('/'),
        "REVERSE SOLIDUS" | "BACKSLASH" => Some('\\'),
        "QUOTATION MARK" => Some('"'),
        "APOSTROPHE" => Some('\''),
        "LATIN SMALL LETTER N" => Some('n'),
        "LATIN SMALL LETTER T" => Some('t'),
        "LATIN SMALL LETTER R" => Some('r'),
        "LATIN SMALL LETTER E" => Some('e'),
        "HORIZONTAL TABULATION" => Some('\t'),
        "LINE FEED" => Some('\n'),
        "CARRIAGE RETURN" => Some('\r'),
        "BELL" => Some('\x07'),
        "BACKSPACE" => Some('\x08'),
        "FORM FEED" => Some('\x0C'),
        "VERTICAL TABULATION" => Some('\x0B'),
        "LATIN SMALL LETTER SHARP S" => Some('\u{00DF}'),
        "MICRO SIGN" => Some('\u{00B5}'),
        "DEGREE SIGN" => Some('\u{00B0}'),
        "PLUS-MINUS SIGN" => Some('\u{00B1}'),
        "EURO SIGN" => Some('\u{20AC}'),
        "POUND SIGN" => Some('\u{00A3}'),
        "YEN SIGN" => Some('\u{00A5}'),
        "BLACK HEART SUIT" => Some('\u{2665}'),
        "BLACK SPADE SUIT" => Some('\u{2660}'),
        "BLACK CLUB SUIT" => Some('\u{2663}'),
        "BLACK DIAMOND SUIT" => Some('\u{2666}'),
        "SMILEY FACE" => Some('\u{263A}'),
        _ => None,
    }
}

/// Process all Python escape sequences in a regular string literal.
/// Handles: `\\`, `\'`, `\"`, `\n`, `\t`, `\r`, `\a`, `\b`, `\f`, `\v`, `\0`,
/// `\ooo` (octal), `\xHH`, `\uXXXX`, `\UXXXXXXXX`, `\N{name}`.
pub(crate) fn apply_escape_sequences(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        match chars.peek().copied() {
            Some('\\') => { chars.next(); result.push('\\'); }
            Some('\'') => { chars.next(); result.push('\''); }
            Some('"') => { chars.next(); result.push('"'); }
            Some('n') => { chars.next(); result.push('\n'); }
            Some('t') => { chars.next(); result.push('\t'); }
            Some('r') => { chars.next(); result.push('\r'); }
            Some('a') => { chars.next(); result.push('\x07'); }
            Some('b') => { chars.next(); result.push('\x08'); }
            Some('f') => { chars.next(); result.push('\x0C'); }
            Some('v') => { chars.next(); result.push('\x0B'); }
            Some('0') => { chars.next(); result.push('\0'); }
            Some('N') => {
                chars.next();
                if chars.peek() == Some(&'{') {
                    chars.next();
                    let mut name = String::new();
                    while let Some(&nc) = chars.peek() {
                        if nc == '}' { chars.next(); break; }
                        name.push(nc);
                        chars.next();
                    }
                    if let Some(uc) = unicode_name_to_char(&name) {
                        result.push(uc);
                    } else {
                        result.push('\\');
                        result.push('N');
                        result.push('{');
                        result.push_str(&name);
                        result.push('}');
                    }
                } else {
                    result.push('\\');
                    result.push('N');
                }
            }
            Some('u') => {
                chars.next();
                let mut hex = String::with_capacity(4);
                for _ in 0..4 { if let Some(h) = chars.next() { hex.push(h); } }
                if let Ok(n) = u32::from_str_radix(&hex, 16) {
                    if let Some(uc) = char::from_u32(n) { result.push(uc); continue; }
                }
                result.push('\\'); result.push('u'); result.push_str(&hex);
            }
            Some('U') => {
                chars.next();
                let mut hex = String::with_capacity(8);
                for _ in 0..8 { if let Some(h) = chars.next() { hex.push(h); } }
                if let Ok(n) = u32::from_str_radix(&hex, 16) {
                    if let Some(uc) = char::from_u32(n) { result.push(uc); continue; }
                }
                result.push('\\'); result.push('U'); result.push_str(&hex);
            }
            Some('x') => {
                chars.next();
                let mut hex = String::with_capacity(2);
                for _ in 0..2 { if let Some(h) = chars.next() { hex.push(h); } }
                if let Ok(n) = u32::from_str_radix(&hex, 16) {
                    if let Some(uc) = char::from_u32(n) { result.push(uc); continue; }
                }
                result.push('\\'); result.push('x'); result.push_str(&hex);
            }
            Some(d) if d.is_ascii_digit() && d != '8' && d != '9' => {
                // Octal escape: \ooo (up to 3 octal digits)
                chars.next();
                let mut oct = String::with_capacity(3);
                oct.push(d);
                for _ in 0..2 {
                    if let Some(&nd) = chars.peek() {
                        if nd.is_ascii_digit() && nd != '8' && nd != '9' {
                            oct.push(nd);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                if let Ok(n) = u32::from_str_radix(&oct, 8) {
                    if let Some(uc) = char::from_u32(n) { result.push(uc); continue; }
                }
                result.push('\\'); result.push_str(&oct);
            }
            _ => result.push(c),
        }
    }
    result
}

/// Process escape sequences in a byte string literal, producing Vec<u8>.
/// Supports: `\\`, `\'`, `\"`, `\n`, `\t`, `\r`, `\a`, `\b`, `\f`, `\v`, `\0`,
/// `\ooo` (octal), `\xHH`. Does NOT support `\u`, `\U`, `\N{name}`.
pub(crate) fn apply_bytes_escapes(s: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            // Only allow ASCII bytes in byte strings
            if c.is_ascii() {
                result.push(c as u8);
            } else {
                // Non-ASCII: encode as UTF-8 bytes
                let mut buf = [0u8; 4];
                let encoded = c.encode_utf8(&mut buf);
                result.extend_from_slice(encoded.as_bytes());
            }
            continue;
        }
        match chars.peek().copied() {
            Some('\\') => { chars.next(); result.push(b'\\'); }
            Some('\'') => { chars.next(); result.push(b'\''); }
            Some('"') => { chars.next(); result.push(b'"'); }
            Some('n') => { chars.next(); result.push(b'\n'); }
            Some('t') => { chars.next(); result.push(b'\t'); }
            Some('r') => { chars.next(); result.push(b'\r'); }
            Some('a') => { chars.next(); result.push(0x07); }
            Some('b') => { chars.next(); result.push(0x08); }
            Some('f') => { chars.next(); result.push(0x0C); }
            Some('v') => { chars.next(); result.push(0x0B); }
            Some('0') => { chars.next(); result.push(0); }
            Some('x') => {
                chars.next();
                let mut hex = String::with_capacity(2);
                for _ in 0..2 { if let Some(h) = chars.next() { hex.push(h); } }
                if let Ok(n) = u8::from_str_radix(&hex, 16) {
                    result.push(n);
                } else {
                    result.push(b'\\');
                    result.push(b'x');
                    result.extend_from_slice(hex.as_bytes());
                }
            }
            Some(d) if d.is_ascii_digit() && d != '8' && d != '9' => {
                chars.next();
                let mut oct = String::with_capacity(3);
                oct.push(d);
                for _ in 0..2 {
                    if let Some(&nd) = chars.peek() {
                        if nd.is_ascii_digit() && nd != '8' && nd != '9' {
                            oct.push(nd);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                if let Ok(n) = u8::from_str_radix(&oct, 8) {
                    result.push(n);
                } else {
                    result.push(b'\\');
                    result.extend_from_slice(oct.as_bytes());
                }
            }
            _ => {
                result.push(b'\\');
                if let Some(nc) = chars.next() {
                    if nc.is_ascii() { result.push(nc as u8); }
                }
            }
        }
    }
    result
}

// --- Callback functions for triple-quoted strings ---

fn lex_triple_dquote(lex: &mut logos::Lexer<TokenKind>) -> Option<String> {
    let remainder = lex.remainder();
    let bytes = remainder.as_bytes();
    let mut i = 0;
    while i < remainder.len() {
        if i + 2 < remainder.len() && bytes[i] == b'"' && bytes[i + 1] == b'"' && bytes[i + 2] == b'"' {
            let content = remainder[..i].to_string();
            lex.bump(i + 3);
            return Some(content);
        }
        if bytes[i] == b'\\' && i + 1 < remainder.len() {
            i += 2;
        } else {
            i += 1;
        }
    }
    None
}

fn lex_triple_squote(lex: &mut logos::Lexer<TokenKind>) -> Option<String> {
    let remainder = lex.remainder();
    let bytes = remainder.as_bytes();
    let mut i = 0;
    while i < remainder.len() {
        if i + 2 < remainder.len() && bytes[i] == b'\'' && bytes[i + 1] == b'\'' && bytes[i + 2] == b'\'' {
            let content = remainder[..i].to_string();
            lex.bump(i + 3);
            return Some(content);
        }
        if bytes[i] == b'\\' && i + 1 < remainder.len() {
            i += 2;
        } else {
            i += 1;
        }
    }
    None
}

// Raw triple-quoted variants (#1678): scan to the matching `"""`/`'''` but
// pass backslashes through literally (no escape processing).
fn lex_raw_triple_dquote(lex: &mut logos::Lexer<TokenKind>) -> Option<String> {
    let remainder = lex.remainder();
    let bytes = remainder.as_bytes();
    let mut i = 0;
    while i + 2 < remainder.len() {
        if bytes[i] == b'"' && bytes[i + 1] == b'"' && bytes[i + 2] == b'"' {
            let content = remainder[..i].to_string();
            lex.bump(i + 3);
            return Some(content);
        }
        i += 1;
    }
    None
}

fn lex_raw_triple_squote(lex: &mut logos::Lexer<TokenKind>) -> Option<String> {
    let remainder = lex.remainder();
    let bytes = remainder.as_bytes();
    let mut i = 0;
    while i + 2 < remainder.len() {
        if bytes[i] == b'\'' && bytes[i + 1] == b'\'' && bytes[i + 2] == b'\'' {
            let content = remainder[..i].to_string();
            lex.bump(i + 3);
            return Some(content);
        }
        i += 1;
    }
    None
}

// --- PEP 701 f-string callbacks ---

/// Lex a double-quoted f-string (PEP 701): called after `f"` is consumed.
fn lex_fstr_dquote(lex: &mut logos::Lexer<TokenKind>) -> Option<String> {
    lex_fstr_inner(lex, b'"')
}

/// Lex a single-quoted f-string (PEP 701): called after `f'` is consumed.
fn lex_fstr_squote(lex: &mut logos::Lexer<TokenKind>) -> Option<String> {
    lex_fstr_inner(lex, b'\'')
}

/// Shared f-string lexer. `close_quote` is `b'"'` or `b'\''`.
///
/// Stack entries: `b'e'` = inside `{expr}`, `b'"'` = double-quoted nested
/// string, `b'\''` = single-quoted nested string.  Pushing `b'e'` on `{`
/// even from inside a nested string layer handles deeply nested f-strings
/// such as `f"{f"{f"deep"}"}"`.
fn lex_fstr_inner(lex: &mut logos::Lexer<TokenKind>, close_quote: u8) -> Option<String> {
    let remainder = lex.remainder();
    let bytes = remainder.as_bytes();
    let mut i = 0usize;
    let mut stack: Vec<u8> = Vec::new();

    while i < remainder.len() {
        let b = bytes[i];
        if stack.is_empty() {
            // ── f-string body ─────────────────────────────────────────────────────────────
            if b == close_quote {
                let content = remainder[..i].to_string();
                lex.bump(i + 1);
                return Some(content);
            } else if b == b'{' {
                if i + 1 < remainder.len() && bytes[i + 1] == b'{' {
                    i += 2; // {{ escape
                } else {
                    stack.push(b'e');
                    i += 1;
                }
            } else if b == b'}' {
                if i + 1 < remainder.len() && bytes[i + 1] == b'}' {
                    i += 2; // }} escape
                } else {
                    i += 1;
                }
            } else if b == b'\\' {
                i += if i + 1 < remainder.len() { 2 } else { 1 };
            } else {
                i += 1;
            }
        } else {
            let top = *stack.last().unwrap();
            if top == b'e' {
                // ── inside expression ─────────────────────────────────────────────────
                if b == b'}' {
                    stack.pop();
                    i += 1;
                } else if b == b'{' {
                    stack.push(b'e');
                    i += 1;
                } else if b == b'"' {
                    stack.push(b'"');
                    i += 1;
                } else if b == b'\'' {
                    stack.push(b'\'');
                    i += 1;
                } else if b == b'\\' {
                    // PEP 701: backslashes allowed in expressions
                    i += if i + 1 < remainder.len() { 2 } else { 1 };
                } else {
                    i += 1;
                }
            } else {
                // ── inside nested string ────────────────────────────────────────────────
                // `{` inside a plain string literal (e.g. `'{'` in `f"{'{'}"`)
                // is NOT an expression start.  Only the outer expression-level
                // `{...}` braces need to be tracked.  The original code that
                // pushed `b'e'` here was incorrect for PEP-701 (Python 3.12+).
                if b == top {
                    stack.pop();
                    i += 1;
                } else if b == b'\\' {
                    i += if i + 1 < remainder.len() { 2 } else { 1 };
                } else {
                    i += 1;
                }
            }
        }
    }
    None // unterminated
}

/// Token kind produced by the lexer.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t]+")]
#[logos(skip r"\\\r?\n[ \t]*")]
pub enum TokenKind {
    // --- Keywords: Control Flow ---
    #[token("def")] Def,
    #[token("return")] Return,
    #[token("if")] If,
    #[token("elif")] Elif,
    #[token("else")] Else,
    #[token("while")] While,
    #[token("for")] For,
    #[token("in")] In,
    #[token("class")] Class,
    #[token("enum")] Enum,
    #[token("match")] Match,
    #[token("case")] Case,
    #[token("import")] Import,
    #[token("from")] From,
    #[token("as")] As,
    #[token("and")] And,
    #[token("or")] Or,
    #[token("not")] Not,
    #[token("True")] True,
    #[token("False")] False,
    #[token("None")] None_,
    #[token("pass")] Pass,
    #[token("break")] Break,
    #[token("continue")] Continue,
    #[token("self")] Self_,

    // --- Keywords: Exception Handling (#206) ---
    #[token("try")] Try,
    #[token("except")] Except,
    #[token("finally")] Finally,
    #[token("raise")] Raise,
    #[token("with")] With,

    // --- Keywords: Async (#207) ---
    #[token("async")] Async,
    #[token("await")] Await,
    #[token("yield")] Yield,

    // --- Keywords: Other (#208, #212) ---
    #[token("lambda")] Lambda,
    #[token("del")] Del,
    #[token("assert")] Assert,
    #[token("global")] Global,
    #[token("nonlocal")] Nonlocal,
    #[token("is")] Is,
    #[token("type")] Type,

    // --- Type keywords ---
    #[token("int")] IntType,
    #[token("float")] FloatType,
    #[token("bool")] BoolType,
    #[token("str")] StrType,
    #[token("list")] ListType,
    #[token("dict")] DictType,
    #[token("tuple")] TupleType,

    // --- Literals ---
    // Complex literals — the exponent form `([eE][+-]?[0-9]+)?` mirrors
    // the sibling Float regexes so `3.5e-20j` / `.5e-2J` / `2E5j` all
    // tokenize as Complex (#1676). Without the exponent, the Float
    // regex consumed the numeric body and `j` fell through to Ident.
    #[regex(r"[0-9]+\.[0-9]*([eE][+-]?[0-9]+)?[jJ]", |lex| {
        let s = lex.slice();
        s[..s.len()-1].parse::<f64>().ok()
    }, priority = 4)]
    #[regex(r"\.[0-9]+([eE][+-]?[0-9]+)?[jJ]", |lex| {
        let s = lex.slice();
        s[..s.len()-1].parse::<f64>().ok()
    }, priority = 4)]
    #[regex(r"[0-9]+([eE][+-]?[0-9]+)?[jJ]", |lex| {
        let s = lex.slice();
        s[..s.len()-1].parse::<f64>().ok()
    }, priority = 3)]
    Complex(f64),
    #[regex(r"[0-9]+\.[0-9]*([eE][+-]?[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    #[regex(r"\.[0-9]+([eE][+-]?[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    #[regex(r"[0-9]+[eE][+-]?[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),
    #[regex(r"0[xX][0-9a-fA-F][0-9a-fA-F_]*", |lex| {
        i64::from_str_radix(&lex.slice()[2..].replace('_', ""), 16).ok()
    }, priority = 3)]
    #[regex(r"0[oO][0-7][0-7_]*", |lex| {
        i64::from_str_radix(&lex.slice()[2..].replace('_', ""), 8).ok()
    }, priority = 3)]
    #[regex(r"0[bB][01][01_]*", |lex| {
        i64::from_str_radix(&lex.slice()[2..].replace('_', ""), 2).ok()
    }, priority = 3)]
    #[regex(r"[0-9][0-9_]*", |lex| lex.slice().replace('_', "").parse::<i64>().ok(), priority = 2)]
    Int(i64),

    // Regular strings
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice(); Some(apply_escape_sequences(&s[1..s.len()-1]))
    })]
    #[regex(r#"'([^'\\]|\\.)*'"#, |lex| {
        let s = lex.slice(); Some(apply_escape_sequences(&s[1..s.len()-1]))
    })]
    Str(String),

    // Triple-quoted strings (#211)
    #[token("\"\"\"", lex_triple_dquote, priority = 10)]
    #[token("'''", lex_triple_squote, priority = 10)]
    TripleStr(String),

    // F-strings with PEP 701 support (#py312):
    //   callback-based lexers handle multiline expressions, quote reuse, and
    //   backslashes in expressions.  Priority 5 beats the Ident regex for `f`.
    #[token("f\"", lex_fstr_dquote, priority = 5)]
    #[token("f'", lex_fstr_squote, priority = 5)]
    FStr(String),

    // Raw f-strings (PEP 498): `rf"..."` / `fr"..."` in either prefix order and
    // either letter case.  The body is scanned exactly like a normal f-string
    // (the same callbacks) — the only difference from `FStr` is that backslash
    // escapes in the literal runs are kept verbatim by the parser.  Priority 6
    // beats both the `Ident` regex (`rf`/`fr` are not names here) and the bare
    // `r"`/`f"` openers, so the 3-char prefix wins the longest match.
    #[token("rf\"", lex_fstr_dquote, priority = 6)]
    #[token("rf'", lex_fstr_squote, priority = 6)]
    #[token("rF\"", lex_fstr_dquote, priority = 6)]
    #[token("rF'", lex_fstr_squote, priority = 6)]
    #[token("Rf\"", lex_fstr_dquote, priority = 6)]
    #[token("Rf'", lex_fstr_squote, priority = 6)]
    #[token("RF\"", lex_fstr_dquote, priority = 6)]
    #[token("RF'", lex_fstr_squote, priority = 6)]
    #[token("fr\"", lex_fstr_dquote, priority = 6)]
    #[token("fr'", lex_fstr_squote, priority = 6)]
    #[token("fR\"", lex_fstr_dquote, priority = 6)]
    #[token("fR'", lex_fstr_squote, priority = 6)]
    #[token("Fr\"", lex_fstr_dquote, priority = 6)]
    #[token("Fr'", lex_fstr_squote, priority = 6)]
    #[token("FR\"", lex_fstr_dquote, priority = 6)]
    #[token("FR'", lex_fstr_squote, priority = 6)]
    RawFStr(String),

    // Raw strings (#212) — no escape processing
    // Raw triple-quoted forms (#1678) registered with priority = 10 so the
    // 4-byte `r"""` / `r'''` opening token beats the 3-byte `r""` / `r''`
    // longest-match of the single-quoted raw regex on input that begins
    // with three quotes.
    #[token("r\"\"\"", lex_raw_triple_dquote, priority = 10)]
    #[token("r'''", lex_raw_triple_squote, priority = 10)]
    // Python raw-string rule: a backslash does NOT introduce an escape (it is
    // kept verbatim in the value), but a backslash-quote pair (`\"` / `\'`)
    // still does not terminate the literal.  The body regex therefore mirrors
    // the regular-string shape (`([^"\\]|\\.)*`) so `\"`/`\'` are consumed as
    // a single non-terminating unit — the callback then slices the content
    // WITHOUT running `apply_escape_sequences`, so the backslash survives.
    #[regex(r#"r"([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice(); Some(s[2..s.len()-1].to_string())
    })]
    #[regex(r#"r'([^'\\]|\\.)*'"#, |lex| {
        let s = lex.slice(); Some(s[2..s.len()-1].to_string())
    })]
    RawStr(String),

    // Byte strings (#212)
    // Bytes triple-quoted forms (#1682). Same shape as the raw-triple-quoted
    // fix in #1678: the 4-byte opener `b"""` / `b'''` beats the 3-byte
    // longest match of the single-quoted byte regex via priority = 10.
    // Re-uses `lex_triple_dquote` / `lex_triple_squote`; downstream bytes
    // processing handles escape sequences from the raw content.
    #[token("b\"\"\"", lex_triple_dquote, priority = 10)]
    #[token("b'''", lex_triple_squote, priority = 10)]
    #[regex(r#"b"([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice(); Some(s[2..s.len()-1].to_string())
    })]
    #[regex(r#"b'([^'\\]|\\.)*'"#, |lex| {
        let s = lex.slice(); Some(s[2..s.len()-1].to_string())
    })]
    // Raw-bytes literal prefixes `br'...'` / `rb'...'` (#1604).
    // Content-wise raw bytes is bytes-with-escapes-kept-literal; the existing
    // b'' regex already passes escapes through unchanged, so the slice just
    // skips the 3-char prefix instead of 2.
    #[regex(r#"br"([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice(); Some(s[3..s.len()-1].to_string())
    })]
    #[regex(r#"br'([^'\\]|\\.)*'"#, |lex| {
        let s = lex.slice(); Some(s[3..s.len()-1].to_string())
    })]
    #[regex(r#"rb"([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice(); Some(s[3..s.len()-1].to_string())
    })]
    #[regex(r#"rb'([^'\\]|\\.)*'"#, |lex| {
        let s = lex.slice(); Some(s[3..s.len()-1].to_string())
    })]
    ByteStr(String),

    // Ellipsis (#212)
    #[token("...")]
    Ellipsis,

    // --- Identifiers ---
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", priority = 1)]
    Ident,

    // --- Operators ---
    #[token("+")] Plus,
    #[token("-")] Minus,
    #[token("*")] Star,
    #[token("/")] Slash,
    #[token("//")] DoubleSlash,
    #[token("%")] Percent,
    #[token("**")] DoubleStar,
    #[token("=")] Eq,
    #[token("==")] EqEq,
    #[token("!=")] NotEq,
    #[token("<")] Lt,
    #[token(">")] Gt,
    #[token("<=")] LtEq,
    #[token(">=")] GtEq,
    #[token("->")] Arrow,
    #[token("|")] Pipe,
    #[token("?")] Question,

    // Augmented assignment (#205)
    #[token("+=")] PlusEq,
    #[token("-=")] MinusEq,
    #[token("*=")] StarEq,
    #[token("/=")] SlashEq,
    #[token("//=")] DoubleSlashEq,
    #[token("%=")] PercentEq,
    #[token("**=")] DoubleStarEq,

    // Bitwise operators (#209)
    #[token("&")] Amp,
    #[token("^")] Caret,
    #[token("~")] Tilde,
    #[token("<<")] LShift,
    #[token(">>")] RShift,
    #[token("&=")] AmpEq,
    #[token("|=")] PipeEq,
    #[token("^=")] CaretEq,
    #[token("<<=")] LShiftEq,
    #[token(">>=")] RShiftEq,

    // Walrus and @ operator (#210)
    #[token(":=")] ColonEq,
    #[token("@")] At,
    #[token("@=")] AtEq,

    // --- Delimiters ---
    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token("[")] LBracket,
    #[token("]")] RBracket,
    #[token("{")] LBrace,
    #[token("}")] RBrace,
    #[token(":")] Colon,
    #[token(",")] Comma,
    #[token(".")] Dot,
    #[token(";")] Semicolon,

    // --- Comments ---
    #[regex(r"#[^\n]*")]
    Comment,

    // --- Newline (significant in Python-like syntax) ---
    #[token("\n")]
    Newline,

    // --- Synthetic tokens (injected by IndentProcessor) ---
    Indent,
    Dedent,
    Eof,
}

impl TokenKind {
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Self::Def | Self::Return | Self::If | Self::Elif | Self::Else
            | Self::While | Self::For | Self::In | Self::Class | Self::Enum
            | Self::Match | Self::Case | Self::Import | Self::From | Self::As
            | Self::And | Self::Or | Self::Not | Self::True | Self::False
            | Self::None_ | Self::Pass | Self::Break | Self::Continue | Self::Self_
            | Self::Try | Self::Except | Self::Finally | Self::Raise | Self::With
            | Self::Async | Self::Await | Self::Yield
            | Self::Lambda | Self::Del | Self::Assert | Self::Global | Self::Nonlocal
            | Self::Is | Self::Type
        )
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Def => f.write_str("def"),
            Self::Return => f.write_str("return"),
            Self::If => f.write_str("if"),
            Self::Elif => f.write_str("elif"),
            Self::Else => f.write_str("else"),
            Self::While => f.write_str("while"),
            Self::For => f.write_str("for"),
            Self::In => f.write_str("in"),
            Self::Class => f.write_str("class"),
            Self::Enum => f.write_str("enum"),
            Self::Match => f.write_str("match"),
            Self::Case => f.write_str("case"),
            Self::Import => f.write_str("import"),
            Self::From => f.write_str("from"),
            Self::As => f.write_str("as"),
            Self::And => f.write_str("and"),
            Self::Or => f.write_str("or"),
            Self::Not => f.write_str("not"),
            Self::True => f.write_str("True"),
            Self::False => f.write_str("False"),
            Self::None_ => f.write_str("None"),
            Self::Pass => f.write_str("pass"),
            Self::Break => f.write_str("break"),
            Self::Continue => f.write_str("continue"),
            Self::Self_ => f.write_str("self"),
            Self::Try => f.write_str("try"),
            Self::Except => f.write_str("except"),
            Self::Finally => f.write_str("finally"),
            Self::Raise => f.write_str("raise"),
            Self::With => f.write_str("with"),
            Self::Async => f.write_str("async"),
            Self::Await => f.write_str("await"),
            Self::Yield => f.write_str("yield"),
            Self::Lambda => f.write_str("lambda"),
            Self::Del => f.write_str("del"),
            Self::Assert => f.write_str("assert"),
            Self::Global => f.write_str("global"),
            Self::Nonlocal => f.write_str("nonlocal"),
            Self::Is => f.write_str("is"),
            Self::Type => f.write_str("type"),
            Self::IntType => f.write_str("int"),
            Self::FloatType => f.write_str("float"),
            Self::BoolType => f.write_str("bool"),
            Self::StrType => f.write_str("str"),
            Self::ListType => f.write_str("list"),
            Self::DictType => f.write_str("dict"),
            Self::TupleType => f.write_str("tuple"),
            Self::Complex(v) => write!(f, "{v}j"),
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::Str(v) => write!(f, "\"{v}\""),
            Self::TripleStr(v) => write!(f, "\"\"\"{v}\"\"\""),
            Self::FStr(v) => write!(f, "f\"{v}\""),
            Self::RawFStr(v) => write!(f, "rf\"{v}\""),
            Self::RawStr(v) => write!(f, "r\"{v}\""),
            Self::ByteStr(v) => write!(f, "b\"{v}\""),
            Self::Ellipsis => f.write_str("..."),
            Self::Ident => f.write_str("identifier"),
            Self::Plus => f.write_str("+"),
            Self::Minus => f.write_str("-"),
            Self::Star => f.write_str("*"),
            Self::Slash => f.write_str("/"),
            Self::DoubleSlash => f.write_str("//"),
            Self::Percent => f.write_str("%"),
            Self::DoubleStar => f.write_str("**"),
            Self::Eq => f.write_str("="),
            Self::EqEq => f.write_str("=="),
            Self::NotEq => f.write_str("!="),
            Self::Lt => f.write_str("<"),
            Self::Gt => f.write_str(">"),
            Self::LtEq => f.write_str("<="),
            Self::GtEq => f.write_str(">="),
            Self::Arrow => f.write_str("->"),
            Self::Pipe => f.write_str("|"),
            Self::Question => f.write_str("?"),
            Self::PlusEq => f.write_str("+="),
            Self::MinusEq => f.write_str("-="),
            Self::StarEq => f.write_str("*="),
            Self::SlashEq => f.write_str("/="),
            Self::DoubleSlashEq => f.write_str("//="),
            Self::PercentEq => f.write_str("%="),
            Self::DoubleStarEq => f.write_str("**="),
            Self::Amp => f.write_str("&"),
            Self::Caret => f.write_str("^"),
            Self::Tilde => f.write_str("~"),
            Self::LShift => f.write_str("<<"),
            Self::RShift => f.write_str(">>"),
            Self::AmpEq => f.write_str("&="),
            Self::PipeEq => f.write_str("|="),
            Self::CaretEq => f.write_str("^="),
            Self::LShiftEq => f.write_str("<<="),
            Self::RShiftEq => f.write_str(">>="),
            Self::ColonEq => f.write_str(":="),
            Self::At => f.write_str("@"),
            Self::AtEq => f.write_str("@="),
            Self::LParen => f.write_str("("),
            Self::RParen => f.write_str(")"),
            Self::LBracket => f.write_str("["),
            Self::RBracket => f.write_str("]"),
            Self::LBrace => f.write_str("{{"),
            Self::RBrace => f.write_str("}}"),
            Self::Colon => f.write_str(":"),
            Self::Comma => f.write_str(","),
            Self::Dot => f.write_str("."),
            Self::Semicolon => f.write_str(";"),
            Self::Comment => f.write_str("comment"),
            Self::Newline => f.write_str("newline"),
            Self::Indent => f.write_str("INDENT"),
            Self::Dedent => f.write_str("DEDENT"),
            Self::Eof => f.write_str("EOF"),
        }
    }
}

/// A token with its kind and byte offset range.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub start: u32,
    pub end: u32,
}

impl Token {
    pub fn new(kind: TokenKind, start: u32, end: u32) -> Self {
        Self { kind, start, end }
    }

    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start as usize..self.end as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    /// Helper: lex source and collect token kinds (skipping errors).
    fn lex_kinds(source: &str) -> Vec<TokenKind> {
        TokenKind::lexer(source)
            .filter_map(|r| r.ok())
            .collect()
    }

    // --- Token creation ---

    #[test]
    fn test_token_new() {
        let tok = Token::new(TokenKind::Def, 0, 3);
        assert_eq!(tok.kind, TokenKind::Def);
        assert_eq!(tok.start, 0);
        assert_eq!(tok.end, 3);
    }

    #[test]
    fn test_token_text() {
        let source = "def foo():";
        let tok = Token::new(TokenKind::Def, 0, 3);
        assert_eq!(tok.text(source), "def");
    }

    #[test]
    fn test_token_clone() {
        let tok = Token::new(TokenKind::Int(42), 0, 2);
        let cloned = tok.clone();
        assert_eq!(tok, cloned);
    }

    // --- Keywords ---

    #[test]
    fn test_lex_all_control_flow_keywords() {
        let kinds = lex_kinds("def return if elif else while for in");
        assert_eq!(kinds, vec![
            TokenKind::Def, TokenKind::Return, TokenKind::If,
            TokenKind::Elif, TokenKind::Else, TokenKind::While,
            TokenKind::For, TokenKind::In,
        ]);
    }

    #[test]
    fn test_lex_class_enum_match() {
        let kinds = lex_kinds("class enum match case");
        assert_eq!(kinds, vec![
            TokenKind::Class, TokenKind::Enum,
            TokenKind::Match, TokenKind::Case,
        ]);
    }

    #[test]
    fn test_lex_import_keywords() {
        let kinds = lex_kinds("import from as");
        assert_eq!(kinds, vec![
            TokenKind::Import, TokenKind::From, TokenKind::As,
        ]);
    }

    #[test]
    fn test_lex_logical_operators() {
        let kinds = lex_kinds("and or not");
        assert_eq!(kinds, vec![
            TokenKind::And, TokenKind::Or, TokenKind::Not,
        ]);
    }

    #[test]
    fn test_lex_bool_none_keywords() {
        let kinds = lex_kinds("True False None");
        assert_eq!(kinds, vec![
            TokenKind::True, TokenKind::False, TokenKind::None_,
        ]);
    }

    #[test]
    fn test_lex_flow_keywords() {
        let kinds = lex_kinds("pass break continue self");
        assert_eq!(kinds, vec![
            TokenKind::Pass, TokenKind::Break,
            TokenKind::Continue, TokenKind::Self_,
        ]);
    }

    #[test]
    fn test_lex_exception_keywords() {
        let kinds = lex_kinds("try except finally raise with");
        assert_eq!(kinds, vec![
            TokenKind::Try, TokenKind::Except, TokenKind::Finally,
            TokenKind::Raise, TokenKind::With,
        ]);
    }

    #[test]
    fn test_lex_async_keywords() {
        let kinds = lex_kinds("async await yield");
        assert_eq!(kinds, vec![
            TokenKind::Async, TokenKind::Await, TokenKind::Yield,
        ]);
    }

    #[test]
    fn test_lex_other_keywords() {
        let kinds = lex_kinds("lambda del assert global nonlocal is type");
        assert_eq!(kinds, vec![
            TokenKind::Lambda, TokenKind::Del, TokenKind::Assert,
            TokenKind::Global, TokenKind::Nonlocal,
            TokenKind::Is, TokenKind::Type,
        ]);
    }

    // --- is_keyword ---

    #[test]
    fn test_is_keyword_true() {
        assert!(TokenKind::Def.is_keyword());
        assert!(TokenKind::Return.is_keyword());
        assert!(TokenKind::If.is_keyword());
        assert!(TokenKind::Try.is_keyword());
        assert!(TokenKind::Async.is_keyword());
        assert!(TokenKind::Lambda.is_keyword());
    }

    #[test]
    fn test_is_keyword_false() {
        assert!(!TokenKind::Plus.is_keyword());
        assert!(!TokenKind::Ident.is_keyword());
        assert!(!TokenKind::Int(42).is_keyword());
        assert!(!TokenKind::LParen.is_keyword());
        assert!(!TokenKind::Comment.is_keyword());
        assert!(!TokenKind::Newline.is_keyword());
    }

    // --- Integer literals ---

    #[test]
    fn test_lex_decimal_int() {
        let kinds = lex_kinds("42");
        assert_eq!(kinds, vec![TokenKind::Int(42)]);
    }

    #[test]
    fn test_lex_zero() {
        let kinds = lex_kinds("0");
        assert_eq!(kinds, vec![TokenKind::Int(0)]);
    }

    #[test]
    fn test_lex_hex_int() {
        let kinds = lex_kinds("0xFF");
        assert_eq!(kinds, vec![TokenKind::Int(255)]);
    }

    #[test]
    fn test_lex_octal_int() {
        let kinds = lex_kinds("0o17");
        assert_eq!(kinds, vec![TokenKind::Int(15)]);
    }

    #[test]
    fn test_lex_binary_int() {
        let kinds = lex_kinds("0b1010");
        assert_eq!(kinds, vec![TokenKind::Int(10)]);
    }

    #[test]
    fn test_lex_int_with_underscores() {
        let kinds = lex_kinds("1_000_000");
        assert_eq!(kinds, vec![TokenKind::Int(1_000_000)]);
    }

    #[test]
    fn test_lex_hex_with_underscores() {
        let kinds = lex_kinds("0xFF_FF");
        assert_eq!(kinds, vec![TokenKind::Int(0xFFFF)]);
    }

    // --- Float literals ---

    #[test]
    fn test_lex_float() {
        let kinds = lex_kinds("3.14");
        assert_eq!(kinds, vec![TokenKind::Float(3.14)]);
    }

    #[test]
    fn test_lex_float_scientific() {
        let kinds = lex_kinds("1e10");
        assert_eq!(kinds, vec![TokenKind::Float(1e10)]);
    }

    #[test]
    fn test_lex_float_scientific_with_decimal() {
        let kinds = lex_kinds("2.5e3");
        assert_eq!(kinds, vec![TokenKind::Float(2.5e3)]);
    }

    #[test]
    fn test_lex_float_negative_exponent() {
        let kinds = lex_kinds("1e-5");
        assert_eq!(kinds, vec![TokenKind::Float(1e-5)]);
    }

    #[test]
    fn test_lex_float_leading_dot() {
        let kinds = lex_kinds(".5");
        assert_eq!(kinds, vec![TokenKind::Float(0.5)]);
    }

    #[test]
    fn test_lex_float_trailing_dot() {
        let kinds = lex_kinds("5.");
        assert_eq!(kinds, vec![TokenKind::Float(5.0)]);
    }

    #[test]
    fn test_lex_float_leading_dot_exp() {
        let kinds = lex_kinds(".5e10");
        assert_eq!(kinds, vec![TokenKind::Float(0.5e10)]);
    }

    #[test]
    fn test_lex_float_trailing_dot_exp() {
        let kinds = lex_kinds("5.e2");
        assert_eq!(kinds, vec![TokenKind::Float(5.0e2)]);
    }

    // --- Complex literals ---

    #[test]
    fn test_lex_complex_int() {
        let kinds = lex_kinds("5j");
        assert_eq!(kinds, vec![TokenKind::Complex(5.0)]);
    }

    #[test]
    fn test_lex_complex_float() {
        let kinds = lex_kinds("3.14j");
        assert_eq!(kinds, vec![TokenKind::Complex(3.14)]);
    }

    #[test]
    fn test_lex_complex_uppercase() {
        let kinds = lex_kinds("2J");
        assert_eq!(kinds, vec![TokenKind::Complex(2.0)]);
    }

    #[test]
    fn test_lex_complex_scientific() {
        // `3.5e-20j` and friends were previously rejected — the
        // Complex regexes lacked the optional exponent (#1676). The
        // Float regex consumed `3.5e-20` and `j` fell through to an
        // identifier, producing a parser error.
        for (src, val) in [
            ("3.5e-20j", 3.5e-20_f64),
            ("2.0E5j", 2.0e5_f64),
            (".5e-2J", 0.005_f64),
            ("1e10j", 1e10_f64),
        ] {
            let kinds = lex_kinds(src);
            assert_eq!(kinds, vec![TokenKind::Complex(val)], "lexing {src}");
        }
    }

    // --- String literals ---

    #[test]
    fn test_lex_double_quoted_string() {
        let kinds = lex_kinds("\"hello\"");
        assert_eq!(kinds, vec![TokenKind::Str("hello".into())]);
    }

    #[test]
    fn test_lex_single_quoted_string() {
        let kinds = lex_kinds("'world'");
        assert_eq!(kinds, vec![TokenKind::Str("world".into())]);
    }

    #[test]
    fn test_lex_string_with_escape() {
        // The lexer processes escape sequences: \" → " at lex time.
        let kinds = lex_kinds("\"he\\\"llo\"");
        assert_eq!(kinds, vec![TokenKind::Str("he\"llo".into())]);
    }

    #[test]
    fn test_lex_empty_string() {
        let kinds = lex_kinds("\"\"");
        assert_eq!(kinds, vec![TokenKind::Str("".into())]);
    }

    // --- F-strings ---

    #[test]
    fn test_lex_fstring_double() {
        let kinds = lex_kinds("f\"hello {name}\"");
        assert_eq!(kinds, vec![TokenKind::FStr("hello {name}".into())]);
    }

    #[test]
    fn test_lex_fstring_single() {
        let kinds = lex_kinds("f'value: {x}'");
        assert_eq!(kinds, vec![TokenKind::FStr("value: {x}".into())]);
    }
    // --- PEP 701 f-string enhancements ---

    #[test]
    fn test_lex_fstring_quote_reuse_single_in_double() {
        // f"result: {d['key']}" — single quotes inside double-quoted expression
        let kinds = lex_kinds("f\"result: {d['key']}\"");
        assert_eq!(kinds, vec![TokenKind::FStr("result: {d['key']}".into())]);
    }

    #[test]
    fn test_lex_fstring_multiline_expr() {
        // f"value: {\n    x + y\n}" — multiline expression
        let src = "f\"value: {\n    x + y\n}\"";
        let kinds = lex_kinds(src);
        assert_eq!(kinds.len(), 1);
        assert!(matches!(&kinds[0], TokenKind::FStr(s) if s.contains("x + y")));
    }

    #[test]
    fn test_lex_fstring_dict_comprehension() {
        // f"{ {k: v for k, v in items} }" — nested braces in expression
        let kinds = lex_kinds("f\"{ {k: v for k, v in items} }\"");
        assert_eq!(kinds, vec![TokenKind::FStr("{ {k: v for k, v in items} }".into())]);
    }

    #[test]
    fn test_lex_fstring_escaped_braces() {
        // f"{{literal}}" — escaped braces should not start an expression
        let kinds = lex_kinds("f\"{{literal}}\"");
        assert_eq!(kinds, vec![TokenKind::FStr("{{literal}}".into())]);
    }

    #[test]
    fn test_lex_raw_fstring_prefix_orders() {
        // Both prefix orders lex to a single RawFStr token, not Ident + string.
        assert_eq!(lex_kinds("rf\"\\n{1}\""), vec![TokenKind::RawFStr("\\n{1}".into())]);
        assert_eq!(lex_kinds("fr\"{1}\\t\""), vec![TokenKind::RawFStr("{1}\\t".into())]);
        assert_eq!(lex_kinds("rf'\\n{1}'"), vec![TokenKind::RawFStr("\\n{1}".into())]);
        assert_eq!(lex_kinds("fr'{1}\\t'"), vec![TokenKind::RawFStr("{1}\\t".into())]);
    }

    #[test]
    fn test_lex_raw_fstring_case_variants() {
        // Upper/mixed-case prefix letters are accepted (Rf/rF/RF/Fr/fR/FR).
        for src in ["Rf\"x\"", "rF\"x\"", "RF\"x\"", "Fr\"x\"", "fR\"x\"", "FR\"x\""] {
            assert_eq!(lex_kinds(src), vec![TokenKind::RawFStr("x".into())], "src={src}");
        }
    }

    // --- Raw strings ---

    #[test]
    fn test_lex_raw_string_double() {
        let kinds = lex_kinds("r\"\\n\\t\"");
        assert_eq!(kinds, vec![TokenKind::RawStr("\\n\\t".into())]);
    }

    #[test]
    fn test_lex_raw_string_single() {
        let kinds = lex_kinds("r'path\\to\\file'");
        assert_eq!(kinds, vec![TokenKind::RawStr("path\\to\\file".into())]);
    }

    #[test]
    fn test_lex_raw_triple_dquote() {
        // #1678 — `r"""...` previously matched the single-quoted regex `r""`
        // and left the rest of the source as garbage.
        let kinds = lex_kinds("r\"\"\"x = b\"hi\" \"\"\"");
        assert_eq!(kinds, vec![TokenKind::RawStr("x = b\"hi\" ".into())]);
    }

    #[test]
    fn test_lex_raw_triple_squote() {
        // Backslashes pass through literally — no escape processing.
        let kinds = lex_kinds("r'''multi\\nline\\t'''");
        assert_eq!(kinds, vec![TokenKind::RawStr("multi\\nline\\t".into())]);
    }

    #[test]
    fn test_lex_raw_triple_multiline() {
        let kinds = lex_kinds("r\"\"\"line1\nline2\\n\"\"\"");
        assert_eq!(kinds, vec![TokenKind::RawStr("line1\nline2\\n".into())]);
    }

    // --- Byte strings ---

    #[test]
    fn test_lex_byte_string() {
        let kinds = lex_kinds("b\"bytes\"");
        assert_eq!(kinds, vec![TokenKind::ByteStr("bytes".into())]);
    }

    #[test]
    fn test_lex_raw_bytes_br_prefix() {
        // `br'...'` is a raw-bytes literal — escapes kept literal.
        let kinds = lex_kinds("br'\\g<a1>'");
        assert_eq!(kinds, vec![TokenKind::ByteStr("\\g<a1>".into())]);
        let kinds = lex_kinds("br\"\\n\"");
        assert_eq!(kinds, vec![TokenKind::ByteStr("\\n".into())]);
    }

    #[test]
    fn test_lex_raw_bytes_rb_prefix() {
        // `rb'...'` — same shape as br'', different prefix order.
        let kinds = lex_kinds("rb'\\d+'");
        assert_eq!(kinds, vec![TokenKind::ByteStr("\\d+".into())]);
        let kinds = lex_kinds("rb\"\\d+\"");
        assert_eq!(kinds, vec![TokenKind::ByteStr("\\d+".into())]);
    }

    #[test]
    fn test_lex_bytes_triple_dquote() {
        // #1682 — `b"""...` previously matched the single-quoted byte regex
        // `b""` and left the rest as garbage.
        let kinds = lex_kinds("b\"\"\"hello world\"\"\"");
        assert_eq!(kinds, vec![TokenKind::ByteStr("hello world".into())]);
    }

    #[test]
    fn test_lex_bytes_triple_squote_multiline() {
        let kinds = lex_kinds("b'''multi\nline'''");
        assert_eq!(kinds, vec![TokenKind::ByteStr("multi\nline".into())]);
    }

    #[test]
    fn test_lex_bytes_triple_with_inner_quote() {
        // Single quote inside b""" is preserved verbatim.
        let kinds = lex_kinds("b\"\"\"has 'quote'\"\"\"");
        assert_eq!(kinds, vec![TokenKind::ByteStr("has 'quote'".into())]);
    }

    // --- Triple-quoted strings ---

    #[test]
    fn test_lex_triple_dquote() {
        let kinds = lex_kinds("\"\"\"multi\nline\"\"\"");
        assert_eq!(kinds, vec![TokenKind::TripleStr("multi\nline".into())]);
    }

    #[test]
    fn test_lex_triple_squote() {
        let kinds = lex_kinds("'''another\nmulti'''");
        assert_eq!(kinds, vec![TokenKind::TripleStr("another\nmulti".into())]);
    }

    #[test]
    fn test_lex_triple_empty() {
        let kinds = lex_kinds("\"\"\"\"\"\"");
        assert_eq!(kinds, vec![TokenKind::TripleStr("".into())]);
    }

    #[test]
    fn test_lex_triple_with_escape() {
        let kinds = lex_kinds("\"\"\"has \\\"\"\" inside\"\"\"");
        assert_eq!(
            kinds,
            vec![TokenKind::TripleStr("has \\\"\"\" inside".into())]
        );
    }

    // --- Ellipsis ---

    #[test]
    fn test_lex_ellipsis() {
        let kinds = lex_kinds("...");
        assert_eq!(kinds, vec![TokenKind::Ellipsis]);
    }

    // --- Operators ---

    #[test]
    fn test_lex_arithmetic_operators() {
        let kinds = lex_kinds("+ - * / // % **");
        assert_eq!(kinds, vec![
            TokenKind::Plus, TokenKind::Minus, TokenKind::Star,
            TokenKind::Slash, TokenKind::DoubleSlash,
            TokenKind::Percent, TokenKind::DoubleStar,
        ]);
    }

    #[test]
    fn test_lex_comparison_operators() {
        let kinds = lex_kinds("== != < > <= >=");
        assert_eq!(kinds, vec![
            TokenKind::EqEq, TokenKind::NotEq,
            TokenKind::Lt, TokenKind::Gt,
            TokenKind::LtEq, TokenKind::GtEq,
        ]);
    }

    #[test]
    fn test_lex_assignment_and_arrow() {
        let kinds = lex_kinds("= ->");
        assert_eq!(kinds, vec![TokenKind::Eq, TokenKind::Arrow]);
    }

    #[test]
    fn test_lex_augmented_assignment() {
        let kinds = lex_kinds("+= -= *= /= //= %= **=");
        assert_eq!(kinds, vec![
            TokenKind::PlusEq, TokenKind::MinusEq,
            TokenKind::StarEq, TokenKind::SlashEq,
            TokenKind::DoubleSlashEq, TokenKind::PercentEq,
            TokenKind::DoubleStarEq,
        ]);
    }

    #[test]
    fn test_lex_bitwise_operators() {
        let kinds = lex_kinds("& ^ ~ << >>");
        assert_eq!(kinds, vec![
            TokenKind::Amp, TokenKind::Caret, TokenKind::Tilde,
            TokenKind::LShift, TokenKind::RShift,
        ]);
    }

    #[test]
    fn test_lex_bitwise_assign() {
        let kinds = lex_kinds("&= |= ^= <<= >>=");
        assert_eq!(kinds, vec![
            TokenKind::AmpEq, TokenKind::PipeEq,
            TokenKind::CaretEq, TokenKind::LShiftEq,
            TokenKind::RShiftEq,
        ]);
    }

    #[test]
    fn test_lex_walrus_and_at() {
        let kinds = lex_kinds(":= @ @=");
        assert_eq!(kinds, vec![
            TokenKind::ColonEq, TokenKind::At, TokenKind::AtEq,
        ]);
    }

    #[test]
    fn test_lex_pipe_and_question() {
        let kinds = lex_kinds("| ?");
        assert_eq!(kinds, vec![TokenKind::Pipe, TokenKind::Question]);
    }

    // --- Delimiters ---

    #[test]
    fn test_lex_delimiters() {
        let kinds = lex_kinds("( ) [ ] { } : , . ;");
        assert_eq!(kinds, vec![
            TokenKind::LParen, TokenKind::RParen,
            TokenKind::LBracket, TokenKind::RBracket,
            TokenKind::LBrace, TokenKind::RBrace,
            TokenKind::Colon, TokenKind::Comma,
            TokenKind::Dot, TokenKind::Semicolon,
        ]);
    }

    // --- Identifiers ---

    #[test]
    fn test_lex_identifier() {
        let kinds = lex_kinds("my_var");
        assert_eq!(kinds, vec![TokenKind::Ident]);
    }

    #[test]
    fn test_lex_identifier_with_numbers() {
        let kinds = lex_kinds("x2");
        assert_eq!(kinds, vec![TokenKind::Ident]);
    }

    #[test]
    fn test_lex_underscore_identifier() {
        let kinds = lex_kinds("_private __dunder__");
        assert_eq!(kinds, vec![TokenKind::Ident, TokenKind::Ident]);
    }

    // --- Comments ---

    #[test]
    fn test_lex_comment() {
        let kinds = lex_kinds("# this is a comment");
        assert_eq!(kinds, vec![TokenKind::Comment]);
    }

    #[test]
    fn test_lex_code_with_comment() {
        let kinds = lex_kinds("42 # answer");
        assert_eq!(kinds, vec![TokenKind::Int(42), TokenKind::Comment]);
    }

    // --- Newline ---

    #[test]
    fn test_lex_newline() {
        let kinds = lex_kinds("a\nb");
        assert_eq!(kinds, vec![
            TokenKind::Ident, TokenKind::Newline, TokenKind::Ident,
        ]);
    }

    // --- Backslash line continuation (PEP 8 / Lib-style explicit join) ---

    #[test]
    fn test_lex_backslash_continuation_skips_newline() {
        // `a \<NL>b` should lex as two idents joined into one logical line
        // (no Newline token between them); the trailing whitespace after \
        // is also consumed.
        let kinds = lex_kinds("a \\\n    b");
        assert_eq!(kinds, vec![TokenKind::Ident, TokenKind::Ident]);
    }

    #[test]
    fn test_lex_backslash_continuation_crlf() {
        // CRLF variant — \\<CR><LF>
        let kinds = lex_kinds("a \\\r\n    b");
        assert_eq!(kinds, vec![TokenKind::Ident, TokenKind::Ident]);
    }

    // --- Type keywords ---

    #[test]
    fn test_lex_type_keywords() {
        let kinds = lex_kinds("int float bool str list dict tuple");
        assert_eq!(kinds, vec![
            TokenKind::IntType, TokenKind::FloatType,
            TokenKind::BoolType, TokenKind::StrType,
            TokenKind::ListType, TokenKind::DictType,
            TokenKind::TupleType,
        ]);
    }

    // --- Display ---

    #[test]
    fn test_display_keywords() {
        assert_eq!(format!("{}", TokenKind::Def), "def");
        assert_eq!(format!("{}", TokenKind::Return), "return");
        assert_eq!(format!("{}", TokenKind::True), "True");
        assert_eq!(format!("{}", TokenKind::False), "False");
        assert_eq!(format!("{}", TokenKind::None_), "None");
    }

    #[test]
    fn test_display_literals() {
        assert_eq!(format!("{}", TokenKind::Int(42)), "42");
        assert_eq!(format!("{}", TokenKind::Float(3.14)), "3.14");
        assert_eq!(format!("{}", TokenKind::Str("hi".into())), "\"hi\"");
        assert_eq!(format!("{}", TokenKind::Complex(1.0)), "1j");
    }

    #[test]
    fn test_display_operators() {
        assert_eq!(format!("{}", TokenKind::Plus), "+");
        assert_eq!(format!("{}", TokenKind::Arrow), "->");
        assert_eq!(format!("{}", TokenKind::EqEq), "==");
        assert_eq!(format!("{}", TokenKind::DoubleStar), "**");
    }

    #[test]
    fn test_display_special() {
        assert_eq!(format!("{}", TokenKind::Ellipsis), "...");
        assert_eq!(format!("{}", TokenKind::Indent), "INDENT");
        assert_eq!(format!("{}", TokenKind::Dedent), "DEDENT");
        assert_eq!(format!("{}", TokenKind::Eof), "EOF");
        assert_eq!(format!("{}", TokenKind::Comment), "comment");
    }

    #[test]
    fn test_display_string_variants() {
        assert_eq!(
            format!("{}", TokenKind::FStr("x".into())),
            "f\"x\""
        );
        assert_eq!(
            format!("{}", TokenKind::RawStr("r".into())),
            "r\"r\""
        );
        assert_eq!(
            format!("{}", TokenKind::ByteStr("b".into())),
            "b\"b\""
        );
        assert_eq!(
            format!("{}", TokenKind::TripleStr("t".into())),
            "\"\"\"t\"\"\""
        );
    }

    // --- Whitespace skipping ---

    #[test]
    fn test_lex_skips_spaces_and_tabs() {
        let kinds = lex_kinds("  42  \t  99  ");
        assert_eq!(kinds, vec![TokenKind::Int(42), TokenKind::Int(99)]);
    }

    // --- Combined expressions ---

    #[test]
    fn test_lex_function_def() {
        let kinds = lex_kinds("def add(a, b):");
        assert_eq!(kinds, vec![
            TokenKind::Def, TokenKind::Ident,
            TokenKind::LParen, TokenKind::Ident,
            TokenKind::Comma, TokenKind::Ident,
            TokenKind::RParen, TokenKind::Colon,
        ]);
    }

    #[test]
    fn test_lex_assignment() {
        let kinds = lex_kinds("x = 42");
        assert_eq!(kinds, vec![
            TokenKind::Ident, TokenKind::Eq, TokenKind::Int(42),
        ]);
    }

    #[test]
    fn test_lex_return_with_annotation() {
        let kinds = lex_kinds("def f() -> int:");
        assert_eq!(kinds, vec![
            TokenKind::Def, TokenKind::Ident,
            TokenKind::LParen, TokenKind::RParen,
            TokenKind::Arrow, TokenKind::IntType, TokenKind::Colon,
        ]);
    }

    #[test]
    fn test_lex_empty_input() {
        let kinds = lex_kinds("");
        assert!(kinds.is_empty());
    }

    // --- unicode_name_to_char ---

    #[test]
    fn test_unicode_name_to_char_snowman() {
        assert_eq!(unicode_name_to_char("SNOWMAN"), Some('\u{2603}'));
    }

    #[test]
    fn test_unicode_name_to_char_copyright() {
        assert_eq!(unicode_name_to_char("COPYRIGHT SIGN"), Some('\u{00A9}'));
    }

    #[test]
    fn test_unicode_name_to_char_latin_a() {
        assert_eq!(unicode_name_to_char("LATIN SMALL LETTER A"), Some('a'));
    }

    #[test]
    fn test_unicode_name_to_char_digit_zero() {
        assert_eq!(unicode_name_to_char("DIGIT ZERO"), Some('0'));
    }

    #[test]
    fn test_unicode_name_to_char_null() {
        assert_eq!(unicode_name_to_char("NULL"), Some('\0'));
    }

    #[test]
    fn test_unicode_name_to_char_space() {
        assert_eq!(unicode_name_to_char("SPACE"), Some(' '));
    }

    #[test]
    fn test_unicode_name_to_char_unknown() {
        assert_eq!(unicode_name_to_char("UNKNOWN NAME XYZ"), None);
    }

    #[test]
    fn test_unicode_name_to_char_empty() {
        assert_eq!(unicode_name_to_char(""), None);
    }

    // --- apply_escape_sequences ---

    #[test]
    fn test_apply_escape_backslash() {
        // Input: "\\" (2 chars: backslash + backslash) → output: "\" (1 backslash)
        assert_eq!(apply_escape_sequences("\\\\"), "\\");
    }

    #[test]
    fn test_apply_escape_single_quote() {
        assert_eq!(apply_escape_sequences("\\'"), "'");
    }

    #[test]
    fn test_apply_escape_double_quote() {
        assert_eq!(apply_escape_sequences("\\\""), "\"");
    }

    #[test]
    fn test_apply_escape_newline() {
        assert_eq!(apply_escape_sequences("\\n"), "\n");
    }

    #[test]
    fn test_apply_escape_tab() {
        assert_eq!(apply_escape_sequences("\\t"), "\t");
    }

    #[test]
    fn test_apply_escape_carriage_return() {
        assert_eq!(apply_escape_sequences("\\r"), "\r");
    }

    #[test]
    fn test_apply_escape_bell() {
        assert_eq!(apply_escape_sequences("\\a"), "\x07");
    }

    #[test]
    fn test_apply_escape_backspace() {
        assert_eq!(apply_escape_sequences("\\b"), "\x08");
    }

    #[test]
    fn test_apply_escape_form_feed() {
        assert_eq!(apply_escape_sequences("\\f"), "\x0C");
    }

    #[test]
    fn test_apply_escape_vertical_tab() {
        assert_eq!(apply_escape_sequences("\\v"), "\x0B");
    }

    #[test]
    fn test_apply_escape_null() {
        assert_eq!(apply_escape_sequences("\\0"), "\0");
    }

    #[test]
    fn test_apply_escape_unicode_name_known() {
        assert_eq!(apply_escape_sequences("\\N{SNOWMAN}"), "\u{2603}");
    }

    #[test]
    fn test_apply_escape_unicode_name_unknown_passthrough() {
        assert_eq!(apply_escape_sequences("\\N{UNKNOWN_XYZ}"), "\\N{UNKNOWN_XYZ}");
    }

    #[test]
    fn test_apply_escape_unicode_name_no_brace() {
        assert_eq!(apply_escape_sequences("\\N"), "\\N");
    }

    #[test]
    fn test_apply_escape_small_u() {
        // \u0041 = 'A'
        assert_eq!(apply_escape_sequences("\\u0041"), "A");
    }

    #[test]
    fn test_apply_escape_large_u() {
        // \U00000041 = 'A'
        assert_eq!(apply_escape_sequences("\\U00000041"), "A");
    }

    #[test]
    fn test_apply_escape_hex() {
        // \x41 = 'A'
        assert_eq!(apply_escape_sequences("\\x41"), "A");
    }

    #[test]
    fn test_apply_escape_octal() {
        // \101 = 0o101 = 65 = 'A'
        assert_eq!(apply_escape_sequences("\\101"), "A");
    }

    #[test]
    fn test_apply_escape_no_escape_passthrough() {
        assert_eq!(apply_escape_sequences("hello"), "hello");
    }
}
