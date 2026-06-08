/// token module for Mamba (#698).
///
/// Implements Python 3.12 `token` stdlib: token type constants, tok_name dict,
/// EXACT_TOKEN_TYPES dict, ISTERMINAL, ISNONTERMINAL, ISEOF functions.
/// All constant values match CPython 3.12 exactly.
use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

dispatch_unary!(dispatch_isterminal, mb_token_isterminal);
dispatch_unary!(dispatch_isnonterminal, mb_token_isnonterminal);
dispatch_unary!(dispatch_iseof, mb_token_iseof);

// ── CPython 3.12 token type constants ──

pub const ENDMARKER: i64 = 0;
pub const NAME: i64 = 1;
pub const NUMBER: i64 = 2;
pub const STRING: i64 = 3;
pub const NEWLINE: i64 = 4;
pub const INDENT: i64 = 5;
pub const DEDENT: i64 = 6;
pub const LPAR: i64 = 7;
pub const RPAR: i64 = 8;
pub const LSQB: i64 = 9;
pub const RSQB: i64 = 10;
pub const COLON: i64 = 11;
pub const COMMA: i64 = 12;
pub const SEMI: i64 = 13;
pub const PLUS: i64 = 14;
pub const MINUS: i64 = 15;
pub const STAR: i64 = 16;
pub const SLASH: i64 = 17;
pub const VBAR: i64 = 18;
pub const AMPER: i64 = 19;
pub const LESS: i64 = 20;
pub const GREATER: i64 = 21;
pub const EQUAL: i64 = 22;
pub const DOT: i64 = 23;
pub const PERCENT: i64 = 24;
pub const LBRACE: i64 = 25;
pub const RBRACE: i64 = 26;
pub const EQEQUAL: i64 = 27;
pub const NOTEQUAL: i64 = 28;
pub const LESSEQUAL: i64 = 29;
pub const GREATEREQUAL: i64 = 30;
pub const TILDE: i64 = 31;
pub const CIRCUMFLEX: i64 = 32;
pub const LEFTSHIFT: i64 = 33;
pub const RIGHTSHIFT: i64 = 34;
pub const DOUBLESTAR: i64 = 35;
pub const PLUSEQUAL: i64 = 36;
pub const MINEQUAL: i64 = 37;
pub const STAREQUAL: i64 = 38;
pub const SLASHEQUAL: i64 = 39;
pub const PERCENTEQUAL: i64 = 40;
pub const AMPEREQUAL: i64 = 41;
pub const VBAREQUAL: i64 = 42;
pub const CIRCUMFLEXEQUAL: i64 = 43;
pub const LEFTSHIFTEQUAL: i64 = 44;
pub const RIGHTSHIFTEQUAL: i64 = 45;
pub const DOUBLESTAREQUAL: i64 = 46;
pub const DOUBLESLASH: i64 = 47;
pub const DOUBLESLASHEQUAL: i64 = 48;
pub const AT: i64 = 49;
pub const ATEQUAL: i64 = 50;
pub const RARROW: i64 = 51;
pub const ELLIPSIS: i64 = 52;
pub const COLONEQUAL: i64 = 53;
pub const EXCLAMATION: i64 = 54;
pub const OP: i64 = 55;
pub const AWAIT: i64 = 56;
pub const ASYNC: i64 = 57;
pub const TYPE_IGNORE: i64 = 58;
pub const TYPE_COMMENT: i64 = 59;
pub const SOFT_KEYWORD: i64 = 60;
pub const FSTRING_START: i64 = 61;
pub const FSTRING_MIDDLE: i64 = 62;
pub const FSTRING_END: i64 = 63;
pub const COMMENT: i64 = 64;
pub const NL: i64 = 65;
pub const ENCODING: i64 = 66;
pub const N_TOKENS: i64 = 67;
pub const ERRORTOKEN: i64 = 68;
pub const NT_OFFSET: i64 = 256;

/// Ordered list of (name, value) for all token constants.
const TOKEN_CONSTANTS: &[(&str, i64)] = &[
    ("ENDMARKER", ENDMARKER),
    ("NAME", NAME),
    ("NUMBER", NUMBER),
    ("STRING", STRING),
    ("NEWLINE", NEWLINE),
    ("INDENT", INDENT),
    ("DEDENT", DEDENT),
    ("LPAR", LPAR),
    ("RPAR", RPAR),
    ("LSQB", LSQB),
    ("RSQB", RSQB),
    ("COLON", COLON),
    ("COMMA", COMMA),
    ("SEMI", SEMI),
    ("PLUS", PLUS),
    ("MINUS", MINUS),
    ("STAR", STAR),
    ("SLASH", SLASH),
    ("VBAR", VBAR),
    ("AMPER", AMPER),
    ("LESS", LESS),
    ("GREATER", GREATER),
    ("EQUAL", EQUAL),
    ("DOT", DOT),
    ("PERCENT", PERCENT),
    ("LBRACE", LBRACE),
    ("RBRACE", RBRACE),
    ("EQEQUAL", EQEQUAL),
    ("NOTEQUAL", NOTEQUAL),
    ("LESSEQUAL", LESSEQUAL),
    ("GREATEREQUAL", GREATEREQUAL),
    ("TILDE", TILDE),
    ("CIRCUMFLEX", CIRCUMFLEX),
    ("LEFTSHIFT", LEFTSHIFT),
    ("RIGHTSHIFT", RIGHTSHIFT),
    ("DOUBLESTAR", DOUBLESTAR),
    ("PLUSEQUAL", PLUSEQUAL),
    ("MINEQUAL", MINEQUAL),
    ("STAREQUAL", STAREQUAL),
    ("SLASHEQUAL", SLASHEQUAL),
    ("PERCENTEQUAL", PERCENTEQUAL),
    ("AMPEREQUAL", AMPEREQUAL),
    ("VBAREQUAL", VBAREQUAL),
    ("CIRCUMFLEXEQUAL", CIRCUMFLEXEQUAL),
    ("LEFTSHIFTEQUAL", LEFTSHIFTEQUAL),
    ("RIGHTSHIFTEQUAL", RIGHTSHIFTEQUAL),
    ("DOUBLESTAREQUAL", DOUBLESTAREQUAL),
    ("DOUBLESLASH", DOUBLESLASH),
    ("DOUBLESLASHEQUAL", DOUBLESLASHEQUAL),
    ("AT", AT),
    ("ATEQUAL", ATEQUAL),
    ("RARROW", RARROW),
    ("ELLIPSIS", ELLIPSIS),
    ("COLONEQUAL", COLONEQUAL),
    ("EXCLAMATION", EXCLAMATION),
    ("OP", OP),
    ("AWAIT", AWAIT),
    ("ASYNC", ASYNC),
    ("TYPE_IGNORE", TYPE_IGNORE),
    ("TYPE_COMMENT", TYPE_COMMENT),
    ("SOFT_KEYWORD", SOFT_KEYWORD),
    ("FSTRING_START", FSTRING_START),
    ("FSTRING_MIDDLE", FSTRING_MIDDLE),
    ("FSTRING_END", FSTRING_END),
    ("COMMENT", COMMENT),
    ("NL", NL),
    ("ENCODING", ENCODING),
    ("N_TOKENS", N_TOKENS),
    ("ERRORTOKEN", ERRORTOKEN),
    ("NT_OFFSET", NT_OFFSET),
];

/// Operator string to token type constant mappings for EXACT_TOKEN_TYPES.
const EXACT_TOKEN_TYPES_DATA: &[(&str, i64)] = &[
    ("(", LPAR),
    (")", RPAR),
    ("[", LSQB),
    ("]", RSQB),
    (":", COLON),
    (",", COMMA),
    (";", SEMI),
    ("+", PLUS),
    ("-", MINUS),
    ("*", STAR),
    ("/", SLASH),
    ("|", VBAR),
    ("&", AMPER),
    ("<", LESS),
    (">", GREATER),
    ("=", EQUAL),
    (".", DOT),
    ("%", PERCENT),
    ("{", LBRACE),
    ("}", RBRACE),
    ("==", EQEQUAL),
    ("!=", NOTEQUAL),
    ("<=", LESSEQUAL),
    (">=", GREATEREQUAL),
    ("~", TILDE),
    ("^", CIRCUMFLEX),
    ("<<", LEFTSHIFT),
    (">>", RIGHTSHIFT),
    ("**", DOUBLESTAR),
    ("+=", PLUSEQUAL),
    ("-=", MINEQUAL),
    ("*=", STAREQUAL),
    ("/=", SLASHEQUAL),
    ("%=", PERCENTEQUAL),
    ("&=", AMPEREQUAL),
    ("|=", VBAREQUAL),
    ("^=", CIRCUMFLEXEQUAL),
    ("<<=", LEFTSHIFTEQUAL),
    (">>=", RIGHTSHIFTEQUAL),
    ("**=", DOUBLESTAREQUAL),
    ("//", DOUBLESLASH),
    ("//=", DOUBLESLASHEQUAL),
    ("@", AT),
    ("@=", ATEQUAL),
    ("->", RARROW),
    ("...", ELLIPSIS),
    (":=", COLONEQUAL),
    ("!", EXCLAMATION),
];

pub fn register() {
    let mut attrs = HashMap::new();
    // Register all integer constants.
    for (name, value) in TOKEN_CONSTANTS {
        attrs.insert(name.to_string(), MbValue::from_int(*value));
    }
    // CPython parity: `tok_name` and `EXACT_TOKEN_TYPES` are dict
    // attributes, not callables. Build them eagerly at register time.
    attrs.insert("tok_name".to_string(), mb_token_tok_name());
    attrs.insert("EXACT_TOKEN_TYPES".to_string(), mb_token_exact_token_types());

    // ISTERMINAL / ISNONTERMINAL / ISEOF are callables.
    let dispatchers: Vec<(&str, usize)> = vec![
        ("ISTERMINAL", dispatch_isterminal as usize),
        ("ISNONTERMINAL", dispatch_isnonterminal as usize),
        ("ISEOF", dispatch_iseof as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("token", attrs);
}

/// token.tok_name — returns dict mapping int token type → name string.
pub fn mb_token_tok_name() -> MbValue {
    use super::super::dict_ops::{mb_dict_new, mb_dict_setitem};
    let dict = mb_dict_new();
    for (name, value) in TOKEN_CONSTANTS {
        let key = MbValue::from_int(*value);
        let val = MbValue::from_ptr(MbObject::new_str(name.to_string()));
        mb_dict_setitem(dict, key, val);
    }
    dict
}

/// token.EXACT_TOKEN_TYPES — returns dict mapping operator string → token type int.
pub fn mb_token_exact_token_types() -> MbValue {
    use super::super::dict_ops::{mb_dict_new, mb_dict_setitem};
    let dict = mb_dict_new();
    for (op, value) in EXACT_TOKEN_TYPES_DATA {
        let key = MbValue::from_ptr(MbObject::new_str(op.to_string()));
        let val = MbValue::from_int(*value);
        mb_dict_setitem(dict, key, val);
    }
    dict
}

/// token.ISTERMINAL(x) -> bool — returns True if x < NT_OFFSET (256).
pub fn mb_token_isterminal(x: MbValue) -> MbValue {
    let v = x.as_int().unwrap_or(0);
    MbValue::from_bool(v < NT_OFFSET)
}

/// token.ISNONTERMINAL(x) -> bool — returns True if x >= NT_OFFSET (256).
pub fn mb_token_isnonterminal(x: MbValue) -> MbValue {
    let v = x.as_int().unwrap_or(0);
    MbValue::from_bool(v >= NT_OFFSET)
}

/// token.ISEOF(x) -> bool — returns True if x == ENDMARKER (0).
pub fn mb_token_iseof(x: MbValue) -> MbValue {
    let v = x.as_int().unwrap_or(-1);
    MbValue::from_bool(v == ENDMARKER)
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: R1, R2
    #[test]
    fn test_constants_values() {
        assert_eq!(NAME, 1, "NAME must be 1");
        assert_eq!(OP, 55, "OP must be 55");
        assert_eq!(N_TOKENS, 67, "N_TOKENS must be 67");
        assert_eq!(ENDMARKER, 0, "ENDMARKER must be 0");
        assert_eq!(NT_OFFSET, 256, "NT_OFFSET must be 256");
        assert_eq!(ERRORTOKEN, 68, "ERRORTOKEN must be 68");
    }

    // REQ: R2
    #[test]
    fn test_isterminal() {
        // NAME=1 is terminal (< 256)
        assert_eq!(
            mb_token_isterminal(MbValue::from_int(NAME)).as_bool(),
            Some(true),
            "ISTERMINAL(NAME) must be true"
        );
        // NT_OFFSET=256 is not terminal
        assert_eq!(
            mb_token_isterminal(MbValue::from_int(NT_OFFSET)).as_bool(),
            Some(false),
            "ISTERMINAL(256) must be false"
        );
        // ERRORTOKEN=68 is terminal
        assert_eq!(
            mb_token_isterminal(MbValue::from_int(ERRORTOKEN)).as_bool(),
            Some(true),
            "ISTERMINAL(ERRORTOKEN) must be true"
        );
    }

    // REQ: R2
    #[test]
    fn test_isnonterminal() {
        // NT_OFFSET=256 is non-terminal
        assert_eq!(
            mb_token_isnonterminal(MbValue::from_int(NT_OFFSET)).as_bool(),
            Some(true),
            "ISNONTERMINAL(256) must be true"
        );
        // NAME=1 is not non-terminal
        assert_eq!(
            mb_token_isnonterminal(MbValue::from_int(NAME)).as_bool(),
            Some(false),
            "ISNONTERMINAL(NAME) must be false"
        );
        // 300 > NT_OFFSET is non-terminal
        assert_eq!(
            mb_token_isnonterminal(MbValue::from_int(300)).as_bool(),
            Some(true),
            "ISNONTERMINAL(300) must be true"
        );
    }

    // REQ: R2
    #[test]
    fn test_iseof() {
        // ENDMARKER=0 is EOF
        assert_eq!(
            mb_token_iseof(MbValue::from_int(ENDMARKER)).as_bool(),
            Some(true),
            "ISEOF(ENDMARKER) must be true"
        );
        // NAME=1 is not EOF
        assert_eq!(
            mb_token_iseof(MbValue::from_int(NAME)).as_bool(),
            Some(false),
            "ISEOF(NAME) must be false"
        );
        // 255 is not EOF
        assert_eq!(
            mb_token_iseof(MbValue::from_int(255)).as_bool(),
            Some(false),
            "ISEOF(255) must be false"
        );
    }

    // REQ: R2
    #[test]
    fn test_tok_name_returns_dict() {
        let dict = mb_token_tok_name();
        // Must return a pointer (i.e., a heap object — the dict).
        assert!(dict.as_ptr().is_some(), "tok_name must return a dict pointer");
    }

    // REQ: R2
    #[test]
    fn test_exact_token_types_returns_dict() {
        let dict = mb_token_exact_token_types();
        assert!(
            dict.as_ptr().is_some(),
            "EXACT_TOKEN_TYPES must return a dict pointer"
        );
    }
}
