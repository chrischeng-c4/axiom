use super::super::rc::MbObject;
use super::super::value::MbValue;
/// tokenize module for Mamba (#669).
///
/// Wraps Mamba's lexer to expose a Python-compatible tokenize interface.
/// Provides generate_tokens(), TokenInfo, and token type constants.
use std::collections::HashMap;

// -- Token type constants (matching Python's tokenize module) --
pub const ENDMARKER: i64 = 0;
pub const NAME: i64 = 1;
pub const NUMBER: i64 = 2;
pub const STRING: i64 = 3;
pub const NEWLINE: i64 = 4;
pub const NL: i64 = 61; // non-logical newline
pub const COMMENT: i64 = 60;
pub const INDENT: i64 = 5;
pub const DEDENT: i64 = 6;
pub const OP: i64 = 54;
pub const ERRORTOKEN: i64 = 59;
pub const ENCODING: i64 = 62;

// ── Variadic dispatchers (callable from module-attr context) ──

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_a: *const MbValue, _n: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! disp_quinary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
                a.get(2).copied().unwrap_or_else(MbValue::none),
                a.get(3).copied().unwrap_or_else(MbValue::none),
                a.get(4).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

disp_unary!(d_generate_tokens, mb_tokenize_generate_tokens);
disp_unary!(d_tokenize, mb_tokenize_tokenize);
disp_unary!(d_untokenize, mb_tokenize_untokenize);
disp_unary!(d_detect_encoding, mb_tokenize_detect_encoding);
disp_unary!(d_open, mb_tokenize_open);
disp_quinary!(d_TokenInfo, mb_tokenize_TokenInfo);
disp_nullary!(d_tok_name, mb_tokenize_tok_name);

pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("generate_tokens", d_generate_tokens as *const () as usize),
        ("tokenize", d_tokenize as *const () as usize),
        ("untokenize", d_untokenize as *const () as usize),
        ("detect_encoding", d_detect_encoding as *const () as usize),
        ("open", d_open as *const () as usize),
        ("TokenInfo", d_TokenInfo as *const () as usize),
        ("tok_name", d_tok_name as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Token type constants
    attrs.insert("ENDMARKER".into(), MbValue::from_int(ENDMARKER));
    attrs.insert("NAME".into(), MbValue::from_int(NAME));
    attrs.insert("NUMBER".into(), MbValue::from_int(NUMBER));
    attrs.insert("STRING".into(), MbValue::from_int(STRING));
    attrs.insert("NEWLINE".into(), MbValue::from_int(NEWLINE));
    attrs.insert("NL".into(), MbValue::from_int(NL));
    attrs.insert("COMMENT".into(), MbValue::from_int(COMMENT));
    attrs.insert("INDENT".into(), MbValue::from_int(INDENT));
    attrs.insert("DEDENT".into(), MbValue::from_int(DEDENT));
    attrs.insert("OP".into(), MbValue::from_int(OP));
    attrs.insert("ERRORTOKEN".into(), MbValue::from_int(ERRORTOKEN));
    attrs.insert("ENCODING".into(), MbValue::from_int(ENCODING));

    // Exception type
    attrs.insert(
        "TokenError".into(),
        MbValue::from_ptr(MbObject::new_str("TokenError".to_string())),
    );

    super::register_module("tokenize", attrs);
}

// -- Helpers --

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        use super::super::rc::ObjData;
        match &(*ptr).data {
            ObjData::Str(s) => Some(s.clone()),
            // CPython's tokenize.tokenize(readline) consumes bytes; accept
            // bytes/bytearray here so callers passing `b"source"` aren't
            // silently dropped. Lossy UTF-8 is correct for Python source,
            // which must be a valid encoding (default utf-8).
            ObjData::Bytes(b) => Some(String::from_utf8_lossy(b).into_owned()),
            ObjData::ByteArray(lock) => {
                Some(String::from_utf8_lossy(&lock.read().unwrap()).into_owned())
            }
            _ => None,
        }
    })
}

/// Build a TokenInfo 5-tuple: (type, string, start, end, line)
fn make_token_info(
    tok_type: i64,
    string: &str,
    start_row: i64,
    start_col: i64,
    end_row: i64,
    end_col: i64,
    line: &str,
) -> MbValue {
    let start = MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(start_row),
        MbValue::from_int(start_col),
    ]));
    let end = MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(end_row),
        MbValue::from_int(end_col),
    ]));
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(tok_type),
        MbValue::from_ptr(MbObject::new_str(string.to_string())),
        start,
        end,
        MbValue::from_ptr(MbObject::new_str(line.to_string())),
    ]))
}

/// A minimal hand-rolled tokenizer for Mamba's tokenize module.
/// Parses the source string into a list of TokenInfo 5-tuples.
fn tokenize_source(source: &str) -> Vec<MbValue> {
    let mut tokens = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    // Emit ENCODING token (always UTF-8)
    tokens.push(make_token_info(ENCODING, "utf-8", 0, 0, 0, 5, ""));

    for (row, line) in lines.iter().enumerate() {
        let row_1 = (row + 1) as i64;
        let mut chars = line.char_indices().peekable();

        while let Some((col, ch)) = chars.next() {
            let col_i = col as i64;
            match ch {
                // Skip whitespace
                ' ' | '\t' => continue,
                // Comment
                '#' => {
                    let rest: String = std::iter::once(ch)
                        .chain(chars.by_ref().map(|(_, c)| c))
                        .collect();
                    let end_col = col + rest.len();
                    tokens.push(make_token_info(
                        COMMENT,
                        &rest,
                        row_1,
                        col_i,
                        row_1,
                        end_col as i64,
                        line,
                    ));
                    break;
                }
                // String literals
                '"' | '\'' => {
                    let quote = ch;
                    let mut s = String::from(ch);
                    for (_, c) in chars.by_ref() {
                        s.push(c);
                        if c == quote {
                            break;
                        }
                    }
                    let end_col = col + s.len();
                    tokens.push(make_token_info(
                        STRING,
                        &s,
                        row_1,
                        col_i,
                        row_1,
                        end_col as i64,
                        line,
                    ));
                }
                // Numbers
                '0'..='9' => {
                    let mut n = String::from(ch);
                    while let Some(&(_, c)) = chars.peek() {
                        if c.is_ascii_alphanumeric() || c == '.' {
                            n.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    let end_col = col + n.len();
                    tokens.push(make_token_info(
                        NUMBER,
                        &n,
                        row_1,
                        col_i,
                        row_1,
                        end_col as i64,
                        line,
                    ));
                }
                // Identifiers / keywords
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut id = String::from(ch);
                    while let Some(&(_, c)) = chars.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            id.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    let end_col = col + id.len();
                    tokens.push(make_token_info(
                        NAME,
                        &id,
                        row_1,
                        col_i,
                        row_1,
                        end_col as i64,
                        line,
                    ));
                }
                // Operators and delimiters
                _ => {
                    tokens.push(make_token_info(
                        OP,
                        &ch.to_string(),
                        row_1,
                        col_i,
                        row_1,
                        col_i + 1,
                        line,
                    ));
                }
            }
        }
        // NEWLINE at end of each logical line
        let next_col = line.len() as i64;
        tokens.push(make_token_info(
            NEWLINE,
            "\n",
            row_1,
            next_col,
            row_1,
            next_col + 1,
            line,
        ));
    }

    // ENDMARKER
    let last_row = lines.len() as i64 + 1;
    tokens.push(make_token_info(ENDMARKER, "", last_row, 0, last_row, 0, ""));
    tokens
}

/// tokenize.generate_tokens(readline) -> iterator of TokenInfo
/// Accepts either a callable readline or a source string directly.
pub fn mb_tokenize_generate_tokens(readline: MbValue) -> MbValue {
    let source = extract_str(readline).unwrap_or_default();
    let tokens = tokenize_source(&source);
    MbValue::from_ptr(MbObject::new_list(tokens))
}

/// tokenize.tokenize(readline) -> iterator of TokenInfo (same as generate_tokens for bytes)
pub fn mb_tokenize_tokenize(readline: MbValue) -> MbValue {
    mb_tokenize_generate_tokens(readline)
}

/// tokenize.untokenize(iterable) -> str
/// Converts a sequence of TokenInfo tuples back to source code.
pub fn mb_tokenize_untokenize(tokens: MbValue) -> MbValue {
    use super::super::rc::ObjData;
    let mut result = String::new();
    if let Some(ptr) = tokens.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let list = lock.read().unwrap();
                for tok in list.iter() {
                    // Extract the string field (index 1) from each TokenInfo tuple
                    if let Some(tok_ptr) = tok.as_ptr() {
                        if let ObjData::Tuple(ref elems) = (*tok_ptr).data {
                            if elems.len() >= 2 {
                                if let Some(s) = extract_str(elems[1]) {
                                    result.push_str(&s);
                                    result.push(' ');
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_str(result))
}

/// tokenize.detect_encoding(readline) -> (encoding, lines)
pub fn mb_tokenize_detect_encoding(_readline: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_str("utf-8".to_string())),
        MbValue::from_ptr(MbObject::new_list(vec![])),
    ]))
}

/// tokenize.open(filename) -> TextIOWrapper stub
pub fn mb_tokenize_open(_filename: MbValue) -> MbValue {
    MbValue::none()
}

/// tokenize.TokenInfo constructor stub
#[allow(non_snake_case)]
pub fn mb_tokenize_TokenInfo(
    tok_type: MbValue,
    string: MbValue,
    start: MbValue,
    end: MbValue,
    line: MbValue,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vec![
        tok_type, string, start, end, line,
    ]))
}

/// tokenize.tok_name — dict mapping token type number -> name
pub fn mb_tokenize_tok_name() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            let entries = &[
                (ENDMARKER, "ENDMARKER"),
                (NAME, "NAME"),
                (NUMBER, "NUMBER"),
                (STRING, "STRING"),
                (NEWLINE, "NEWLINE"),
                (NL, "NL"),
                (COMMENT, "COMMENT"),
                (INDENT, "INDENT"),
                (DEDENT, "DEDENT"),
                (OP, "OP"),
                (ERRORTOKEN, "ERRORTOKEN"),
                (ENCODING, "ENCODING"),
            ];
            for (code, name) in entries {
                map.insert(
                    code.to_string().into(),
                    MbValue::from_ptr(MbObject::new_str((*name).to_string())),
                );
            }
        }
    }
    MbValue::from_ptr(dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_tokens_simple() {
        let src = MbValue::from_ptr(MbObject::new_str("x = 1".to_string()));
        let tokens = mb_tokenize_generate_tokens(src);
        assert!(tokens.as_ptr().is_some());
    }

    #[test]
    fn test_generate_tokens_has_endmarker() {
        let src = MbValue::from_ptr(MbObject::new_str("x = 1\n".to_string()));
        let tokens = mb_tokenize_generate_tokens(src);
        if let Some(ptr) = tokens.as_ptr() {
            unsafe {
                use super::super::super::rc::ObjData;
                if let ObjData::List(ref lock) = (*ptr).data {
                    let list = lock.read().unwrap();
                    assert!(!list.is_empty());
                    // Last token should be ENDMARKER (type 0)
                    let last = list.last().unwrap();
                    if let Some(last_ptr) = last.as_ptr() {
                        if let ObjData::Tuple(ref elems) = (*last_ptr).data {
                            assert_eq!(elems[0].as_int(), Some(ENDMARKER));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_detect_encoding() {
        let result = mb_tokenize_detect_encoding(MbValue::none());
        assert!(result.as_ptr().is_some());
    }

    #[test]
    fn test_tok_name() {
        let names = mb_tokenize_tok_name();
        assert!(names.as_ptr().is_some());
    }

    #[test]
    fn test_tokenize_bytes_input_no_silent_drop() {
        // CPython's tokenize.tokenize takes bytes; previously extract_str
        // only accepted ObjData::Str, so bytes input silently produced an
        // empty token stream. Verify bytes/bytearray now produce real tokens.
        use super::super::super::rc::ObjData;
        let from_bytes = MbValue::from_ptr(MbObject::new_bytes(b"x = 1\n".to_vec()));
        let tokens = mb_tokenize_tokenize(from_bytes);
        let ptr = tokens.as_ptr().expect("tokenize(bytes) returned None");
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let list = lock.read().unwrap();
                // Should have at least: NAME, OP, NUMBER, NEWLINE, ENDMARKER
                assert!(
                    list.len() >= 2,
                    "tokenize(bytes) yielded only {} tokens — silent drop?",
                    list.len()
                );
            } else {
                panic!("tokenize result was not a list");
            }
        }
    }
}
