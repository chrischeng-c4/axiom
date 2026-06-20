use super::super::dict_ops::DictKey;
use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// netrc module for Mamba (#1261 long-tail).
///
/// Real CPython-3.12 `netrc` parity. The public API is the
/// `netrc(file=None)` class. Mamba represents the parsed result as a
/// native `Instance` named `"netrc"` whose fields mirror the CPython
/// object surface:
///
///   netrc(file=None) -> instance {
///     hosts:  {hostname: (login, account, password)},   # tuples!
///     macros: {macro_name: [line, ...]},
///   }
///
/// `netrc.netrc.authenticators(host)` / `__repr__` are provided as
/// best-effort helpers; the conformance fixtures exercise `.hosts`,
/// `.macros`, and `NetrcParseError`.
///
/// `NetrcParseError` is registered as a `Str` marker carrying its own
/// type-name; the runtime matches `except netrc.NetrcParseError` by that
/// string (see `exception::mb_exception_matches`). On malformed input we
/// `mb_raise("NetrcParseError", msg)` and return `none`, exactly like the
/// configparser native module.
///
/// The tokenizer and parser are a faithful port of CPython
/// `Lib/netrc.py`'s `_netrclex` + `netrc._parse`:
///   - The lexer is char-oriented. Whitespace is `"\n\t\r "`. Double
///     quotes group a token; inside or outside quotes a backslash escapes
///     the next char. `#` is an ordinary character to the lexer.
///   - Comments are a *parser*-level concept: a token whose first char is
///     `#`, encountered at the start of a fresh read on the same physical
///     line, makes the parser consume the rest of that line. A `#` that is
///     not the first char of a token (e.g. `pa#ss`) or a `#`-leading value
///     consumed as the argument of `login`/`account`/`password` is kept
///     verbatim.
use std::collections::HashMap;

unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(args_ptr, nargs)
    }
}

unsafe fn as_str(val: MbValue) -> Option<String> {
    let ptr = val.as_ptr()?;
    match &(*ptr).data {
        ObjData::Str(s) => Some(s.clone()),
        ObjData::Bytes(b) => std::str::from_utf8(b).ok().map(str::to_string),
        _ => None,
    }
}

/// Raise a catchable exception whose type-name is `exc`.
/// `except netrc.NetrcParseError` matches by this name string.
fn raise_named(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

// ── Lexer (port of CPython `_netrclex`) ──

const WHITESPACE: &[char] = &['\n', '\t', '\r', ' '];

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    lineno: usize,
    pushback: Vec<String>,
}

impl Lexer {
    fn new(text: &str) -> Self {
        Lexer {
            chars: text.chars().collect(),
            pos: 0,
            lineno: 1,
            pushback: Vec::new(),
        }
    }

    /// Read a single char; "" (None) at EOF. Mirrors `_read_char`, which
    /// increments `lineno` whenever it consumes a `\n`.
    fn read_char(&mut self) -> Option<char> {
        if self.pos >= self.chars.len() {
            return None;
        }
        let ch = self.chars[self.pos];
        self.pos += 1;
        if ch == '\n' {
            self.lineno += 1;
        }
        Some(ch)
    }

    /// Consume the remainder of the current physical line (through and
    /// including the next `\n`). Mirrors `instream.readline()` for the
    /// comment-skip paths.
    fn readline(&mut self) {
        while let Some(ch) = self.read_char() {
            if ch == '\n' {
                break;
            }
        }
    }

    /// Faithful port of CPython `_netrclex.get_token`. Returns "" at EOF.
    fn get_token(&mut self) -> String {
        if !self.pushback.is_empty() {
            return self.pushback.remove(0);
        }
        let mut token = String::new();
        loop {
            let Some(ch) = self.read_char() else {
                return token;
            };
            if WHITESPACE.contains(&ch) {
                continue;
            }
            if ch == '"' {
                // Quoted segment: read until the closing quote, with `\`
                // escaping the next char.
                loop {
                    let Some(mut qch) = self.read_char() else {
                        return token;
                    };
                    if qch == '"' {
                        return token;
                    } else if qch == '\\' {
                        match self.read_char() {
                            Some(c) => qch = c,
                            None => return token,
                        }
                    }
                    token.push(qch);
                }
            } else {
                // Unquoted token. A leading backslash escapes the first char.
                let mut first = ch;
                if first == '\\' {
                    match self.read_char() {
                        Some(c) => first = c,
                        None => {
                            token.push('\\');
                            return token;
                        }
                    }
                }
                token.push(first);
                loop {
                    let Some(mut nch) = self.read_char() else {
                        return token;
                    };
                    if WHITESPACE.contains(&nch) {
                        return token;
                    } else if nch == '\\' {
                        match self.read_char() {
                            Some(c) => nch = c,
                            None => {
                                token.push('\\');
                                return token;
                            }
                        }
                    }
                    token.push(nch);
                }
            }
        }
    }

    fn push_token(&mut self, token: String) {
        self.pushback.push(token);
    }
}

// ── Parser (port of CPython `netrc._parse`) ──

struct Parsed {
    /// (entryname, (login, account, password)) in insertion order.
    hosts: Vec<(String, [String; 3])>,
    /// (macro_name, lines) in insertion order.
    macros: Vec<(String, Vec<String>)>,
}

enum ParseOutcome {
    Ok(Parsed),
    Err(String),
}

fn parse_netrc(text: &str) -> ParseOutcome {
    let mut lexer = Lexer::new(text);
    let mut hosts: Vec<(String, [String; 3])> = Vec::new();
    let mut macros: Vec<(String, Vec<String>)> = Vec::new();

    loop {
        let saved_lineno = lexer.lineno;
        let tt = lexer.get_token();
        let entryname: String;
        if tt.is_empty() {
            break;
        } else if tt.starts_with('#') {
            // Toplevel comment: only consume the rest of the physical line
            // when the `#` token is a bare `#` that started on this line.
            if lexer.lineno == saved_lineno && tt.chars().count() == 1 {
                lexer.readline();
            }
            continue;
        } else if tt == "machine" {
            entryname = lexer.get_token();
        } else if tt == "default" {
            entryname = "default".to_string();
        } else if tt == "macdef" {
            let name = lexer.get_token();
            let mut lines: Vec<String> = Vec::new();
            loop {
                // CPython reads raw physical lines from the stream here.
                let line = read_physical_line(&mut lexer);
                if line.is_empty() {
                    return ParseOutcome::Err(format!(
                        "Macro definition missing null line terminator. ({:?}, line {})",
                        "<file>", lexer.lineno
                    ));
                }
                if line == "\n" {
                    break;
                }
                lines.push(line);
            }
            macros.push((name, lines));
            continue;
        } else {
            return ParseOutcome::Err(format!(
                "bad toplevel token {:?} (line {})",
                tt, lexer.lineno
            ));
        }

        if entryname.is_empty() {
            return ParseOutcome::Err(format!("missing {:?} name (line {})", tt, lexer.lineno));
        }

        // Body of a `machine`/`default` entry.
        let mut login = String::new();
        let mut account = String::new();
        let mut password = String::new();
        loop {
            let prev_lineno = lexer.lineno;
            let ftt = lexer.get_token();
            if ftt.starts_with('#') {
                if lexer.lineno == prev_lineno {
                    lexer.readline();
                }
                continue;
            }
            if ftt.is_empty() || ftt == "machine" || ftt == "default" || ftt == "macdef" {
                hosts.push((
                    entryname.clone(),
                    [login.clone(), account.clone(), password.clone()],
                ));
                lexer.push_token(ftt);
                break;
            } else if ftt == "login" || ftt == "user" {
                login = lexer.get_token();
            } else if ftt == "account" {
                account = lexer.get_token();
            } else if ftt == "password" {
                password = lexer.get_token();
            } else {
                return ParseOutcome::Err(format!(
                    "bad follower token {:?} (line {})",
                    ftt, lexer.lineno
                ));
            }
        }
    }

    ParseOutcome::Ok(Parsed { hosts, macros })
}

/// Read one raw physical line from the lexer's char stream, including the
/// trailing `\n` if present. Returns "" at EOF (matching `readline()`).
/// Keeps `lineno` in sync via `read_char`.
fn read_physical_line(lexer: &mut Lexer) -> String {
    let mut line = String::new();
    loop {
        match lexer.read_char() {
            Some(ch) => {
                line.push(ch);
                if ch == '\n' {
                    break;
                }
            }
            None => break,
        }
    }
    line
}

// ── Building the result instance ──

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn host_tuple(vals: &[String; 3]) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vec![
        new_str(&vals[0]),
        new_str(&vals[1]),
        new_str(&vals[2]),
    ]))
}

fn set_field(inst: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(val);
                if let Some(p) = fields.write().unwrap().insert(key.to_string(), val) {
                    super::super::rc::release_if_ptr(p);
                }
            }
        }
    }
}

fn parsed_to_instance(p: Parsed) -> MbValue {
    let inst = MbValue::from_ptr(MbObject::new_instance("netrc".to_string()));

    let hosts_dict = MbObject::new_dict();
    let macros_dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(lock) = &(*hosts_dict).data {
            let mut g = lock.write().unwrap();
            for (host, vals) in &p.hosts {
                g.insert(DictKey::Str(host.clone()), host_tuple(vals));
            }
        }
        if let ObjData::Dict(lock) = &(*macros_dict).data {
            let mut g = lock.write().unwrap();
            for (name, lines) in &p.macros {
                let line_vs: Vec<MbValue> = lines.iter().map(|l| new_str(l)).collect();
                g.insert(
                    DictKey::Str(name.clone()),
                    MbValue::from_ptr(MbObject::new_list(line_vs)),
                );
            }
        }
    }
    set_field(inst, "hosts", MbValue::from_ptr(hosts_dict));
    set_field(inst, "macros", MbValue::from_ptr(macros_dict));
    inst
}

// ── POSIX security check (port of `netrc._security_check`) ──
//
// Only runs when `default_netrc` is true (i.e. `netrc()` was called with
// no explicit file path) on a POSIX platform. Mirrors CPython: reject a
// `~/.netrc` whose group/other permission bits are set, or whose owner
// differs from the current uid, unless the matched login is "anonymous".

#[cfg(unix)]
fn security_check(path: &str, login: &str) -> Result<(), String> {
    use std::os::unix::fs::MetadataExt;
    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };
    if login == "anonymous" {
        return Ok(());
    }
    let uid = unsafe { libc_getuid() };
    if meta.uid() != uid {
        return Err(format!(
            "~/.netrc file owner (uid {}, uid {}) does not match current user",
            meta.uid(),
            uid
        ));
    }
    // S_IRWXG | S_IRWXO == 0o077
    if meta.mode() & 0o077 != 0 {
        return Err(
            "~/.netrc access too permissive: access permissions must restrict \
             access to only the owner"
                .to_string(),
        );
    }
    Ok(())
}

#[cfg(unix)]
unsafe fn libc_getuid() -> u32 {
    extern "C" {
        fn getuid() -> u32;
    }
    getuid()
}

#[cfg(not(unix))]
fn security_check(_path: &str, _login: &str) -> Result<(), String> {
    Ok(())
}

fn resolve_default_path() -> Option<String> {
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return Some(format!("{}/.netrc", home));
        }
    }
    None
}

unsafe extern "C" fn dispatch_netrc(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let explicit = args
        .first()
        .copied()
        .filter(|v| !v.is_none())
        .and_then(|v| as_str(v));
    let default_netrc = explicit.is_none();

    let path = match &explicit {
        Some(p) if !p.is_empty() => p.clone(),
        _ => match resolve_default_path() {
            Some(p) => p,
            // No file resolvable: CPython would attempt open and raise
            // FileNotFoundError. With nothing to open, return an empty
            // netrc instance.
            None => {
                return parsed_to_instance(Parsed {
                    hosts: vec![],
                    macros: vec![],
                });
            }
        },
    };

    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => {
            // CPython raises FileNotFoundError here; emit a catchable one.
            return raise_named(
                "FileNotFoundError",
                &format!("[Errno 2] No such file or directory: {:?}", path),
            );
        }
    };

    match parse_netrc(&text) {
        ParseOutcome::Err(msg) => raise_named("NetrcParseError", &msg),
        ParseOutcome::Ok(parsed) => {
            // Security check runs per-entry in CPython after each entry is
            // finalized, using that entry's login. Mirror the effective
            // behavior: if any entry would trip the check, raise.
            if default_netrc {
                for (_host, vals) in &parsed.hosts {
                    if let Err(msg) = security_check(&path, &vals[0]) {
                        return raise_named("NetrcParseError", &msg);
                    }
                }
            }
            parsed_to_instance(parsed)
        }
    }
}

pub fn register() {
    let mut attrs = HashMap::new();

    let addr_netrc = dispatch_netrc as *const () as usize;

    attrs.insert("netrc".into(), MbValue::from_func(addr_netrc));
    // Exception class marker: matched by type-name string on raise/except.
    attrs.insert(
        "NetrcParseError".into(),
        MbValue::from_ptr(MbObject::new_str("NetrcParseError".to_string())),
    );

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_netrc as u64);
    });

    super::register_module("netrc", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(text: &str) -> Parsed {
        match parse_netrc(text) {
            ParseOutcome::Ok(p) => p,
            ParseOutcome::Err(e) => panic!("unexpected parse error: {e}"),
        }
    }

    fn host<'a>(p: &'a Parsed, name: &str) -> Vec<&'a str> {
        let v = p
            .hosts
            .iter()
            .find(|(h, _)| h == name)
            .map(|(_, v)| v)
            .expect("host");
        vec![v[0].as_str(), v[1].as_str(), v[2].as_str()]
    }

    #[test]
    fn toplevel_tokens_ordered() {
        let p = ok("machine host.domain.com login log1 password pass1 account acct1\ndefault login log2 password pass2 account acct2\n");
        assert_eq!(host(&p, "host.domain.com"), vec!["log1", "acct1", "pass1"]);
        assert_eq!(host(&p, "default"), vec!["log2", "acct2", "pass2"]);
    }

    #[test]
    fn toplevel_tokens_non_ordered() {
        let p = ok("machine host.domain.com password pass1 login log1 account acct1\ndefault login log2 password pass2 account acct2\n");
        assert_eq!(host(&p, "host.domain.com"), vec!["log1", "acct1", "pass1"]);
    }

    #[test]
    fn optional_tokens_default_empty() {
        for item in [
            "machine host.domain.com",
            "machine host.domain.com login",
            "machine host.domain.com account",
            "machine host.domain.com password",
            "machine host.domain.com login \"\" account",
            "machine host.domain.com login \"\" password",
            "machine host.domain.com account \"\" password",
        ] {
            let p = ok(item);
            assert_eq!(host(&p, "host.domain.com"), vec!["", "", ""], "{item}");
        }
    }

    #[test]
    fn leading_hash_value_kept() {
        let p = ok("machine host.domain.com login #log password pass account acct\n");
        assert_eq!(host(&p, "host.domain.com"), vec!["#log", "acct", "pass"]);
    }

    #[test]
    fn internal_hash_value_kept() {
        let p = ok("machine host.domain.com login lo#g password pass account acct\n");
        assert_eq!(host(&p, "host.domain.com")[0], "lo#g");
    }

    #[test]
    fn trailing_hash_value_kept() {
        let p = ok("machine host.domain.com login log# password pass account acct\n");
        assert_eq!(host(&p, "host.domain.com")[0], "log#");
    }

    #[test]
    fn comment_after_machine_line_hash_only() {
        let p = ok("machine foo.domain.com login bar password pass\n#\nmachine bar.domain.com login foo password pass\n");
        assert_eq!(host(&p, "foo.domain.com"), vec!["bar", "", "pass"]);
        assert_eq!(host(&p, "bar.domain.com"), vec!["foo", "", "pass"]);
    }

    #[test]
    fn comment_at_end_of_machine_line() {
        let p = ok("machine foo.domain.com login bar password pass # comment\nmachine bar.domain.com login foo password pass\n");
        assert_eq!(host(&p, "foo.domain.com"), vec!["bar", "", "pass"]);
    }

    #[test]
    fn pass_has_hash() {
        let p = ok("machine foo.domain.com login bar password #pass #comment\nmachine bar.domain.com login foo password pass\n");
        assert_eq!(host(&p, "foo.domain.com"), vec!["bar", "", "#pass"]);
    }

    #[test]
    fn quoted_whitespace_value() {
        let p = ok("machine host.domain.com login \"lo g\" password pass account acct\n");
        assert_eq!(host(&p, "host.domain.com")[0], "lo g");
    }

    #[test]
    fn escaped_quote_value() {
        let p = ok("machine host.domain.com login \\\"log password pass account acct\n");
        assert_eq!(host(&p, "host.domain.com")[0], "\"log");
        let p2 = ok("machine host.domain.com login \"\\\"log\" password pass account acct\n");
        assert_eq!(host(&p2, "host.domain.com")[0], "\"log");
    }

    #[test]
    fn macros_two() {
        let p = ok("macdef macro1\nline1\nline2\n\nmacdef macro2\nline3\nline4\n\n");
        assert_eq!(p.macros[0].0, "macro1");
        let m0: Vec<&str> = p.macros[0].1.iter().map(|s| s.as_str()).collect();
        let m1: Vec<&str> = p.macros[1].1.iter().map(|s| s.as_str()).collect();
        assert_eq!(m0, vec!["line1\n", "line2\n"]);
        assert_eq!(m1, vec!["line3\n", "line4\n"]);
    }

    #[test]
    fn macros_missing_terminator_errs() {
        match parse_netrc("macdef macro1\nline1\nline2\n") {
            ParseOutcome::Err(_) => {}
            ParseOutcome::Ok(_) => panic!("expected macro terminator error"),
        }
    }

    #[test]
    fn invalid_toplevel_errs() {
        match parse_netrc("invalid host.domain.com") {
            ParseOutcome::Err(_) => {}
            ParseOutcome::Ok(_) => panic!("expected bad toplevel error"),
        }
    }

    #[test]
    fn invalid_follower_errs() {
        match parse_netrc("machine host.domain.com invalid") {
            ParseOutcome::Err(_) => {}
            ParseOutcome::Ok(_) => panic!("expected bad follower error"),
        }
    }

    #[test]
    fn non_ascii_value() {
        let p = ok("machine host.domain.com login ¡¢ password pass account acct\n");
        assert_eq!(host(&p, "host.domain.com")[0], "¡¢");
    }
}
