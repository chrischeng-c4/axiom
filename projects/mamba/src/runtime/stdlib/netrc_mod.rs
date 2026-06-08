use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// netrc module for Mamba (#1261 long-tail).
///
/// Replaces the long_tail stub which exposed `netrc` / `NetrcParseError`
/// as bare class shells (returning `{}`) with a real parser. The
/// CPython public API is the `netrc(file=None)` class whose
/// `.authenticators(host)` method picks the right login tuple. Mamba
/// doesn't yet support bound-method dispatch on returned instances, so
/// we expose the parsed data as a dict (mirroring `argparse_mod`'s
/// dict-as-instance pattern):
///
///   netrc(path=None) -> dict {
///     "hosts":   {hostname: [login, account, password]},
///     "macros":  {macro_name: [line, ...]},
///     "default": [login, account, password]  // or absent if no `default` block
///   }
///
/// Path resolution: `path` arg overrides everything; otherwise we
/// try `$NETRC`, then `~/.netrc` (POSIX). Missing file -> empty
/// dict with `hosts={}` / `macros={}` (no exception, since we can't
/// raise FileNotFoundError out of a native dispatcher in a way the
/// runtime would re-catch).
///
/// Parser format (RFC-less; follows CPython `Lib/netrc.py`):
///   - Tokens: whitespace-separated; double quotes group a token.
///     Backslash before " is an escape.
///   - Top-level keywords: `machine <name>`, `default`, `macdef <name>`.
///   - Inside `machine` / `default` blocks: `login X`, `password X`,
///     `account X`. Unknown keywords terminate the current block.
///   - `macdef` body: every line until a blank line is appended as
///     raw text (no token splitting).
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

fn resolve_path(explicit: Option<String>) -> Option<String> {
    if let Some(p) = explicit {
        if !p.is_empty() {
            return Some(p);
        }
    }
    if let Ok(p) = std::env::var("NETRC") {
        if !p.is_empty() {
            return Some(p);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return Some(format!("{}/.netrc", home));
        }
    }
    None
}

/// Tokenize one logical line of netrc content. Handles double-quoted
/// strings with `\"` and `\\` escapes; everything outside quotes is
/// whitespace-split.
fn tokenize_line(line: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    let mut escape = false;
    for ch in line.chars() {
        if escape {
            cur.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' && in_quote {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_quote = !in_quote;
            continue;
        }
        if ch.is_whitespace() && !in_quote {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            continue;
        }
        cur.push(ch);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

struct Parsed {
    hosts: Vec<(String, [Option<String>; 3])>,
    default: Option<[Option<String>; 3]>,
    macros: Vec<(String, Vec<String>)>,
}

fn parse_netrc(text: &str) -> Parsed {
    // CPython's parser is line-oriented for `macdef` bodies and
    // token-oriented for everything else. We mirror that: iterate
    // lines, and within "normal" context tokenize the line + carry
    // tokens across lines.
    let mut hosts: Vec<(String, [Option<String>; 3])> = Vec::new();
    let mut default: Option<[Option<String>; 3]> = None;
    let mut macros: Vec<(String, Vec<String>)> = Vec::new();

    enum State {
        Top,
        Macdef(String, Vec<String>),
    }
    let mut state = State::Top;

    // Carry-over token stream when we're inside a `machine`/`default`
    // block but the current line is exhausted. We accumulate tokens
    // line-by-line so a single `machine X login Y` may span lines.
    let mut tokens: Vec<String> = Vec::new();
    let mut active: Option<(Option<String>, [Option<String>; 3])> = None;
    // active = Some((Some(host), [login, account, password]))   -- machine block
    // active = Some((None,       [login, account, password]))   -- default block

    fn flush(
        active: &mut Option<(Option<String>, [Option<String>; 3])>,
        hosts: &mut Vec<(String, [Option<String>; 3])>,
        default: &mut Option<[Option<String>; 3]>,
    ) {
        if let Some((host_opt, vals)) = active.take() {
            if let Some(host) = host_opt {
                hosts.push((host, vals));
            } else {
                *default = Some(vals);
            }
        }
    }

    for raw_line in text.lines() {
        match &mut state {
            State::Macdef(name, lines) => {
                if raw_line.trim().is_empty() {
                    // Blank line terminates the macdef body.
                    let n = std::mem::take(name);
                    let l = std::mem::take(lines);
                    macros.push((n, l));
                    state = State::Top;
                    continue;
                }
                lines.push(raw_line.to_string());
                continue;
            }
            State::Top => {}
        }

        // Strip `#` comments — CPython's netrc doesn't strictly
        // support them but real-world files often have them.
        let stripped = match raw_line.find('#') {
            Some(idx) => &raw_line[..idx],
            None => raw_line,
        };
        tokens.extend(tokenize_line(stripped));

        // Consume tokens by walking the stream.
        let mut i = 0;
        while i < tokens.len() {
            let tok = &tokens[i];
            match tok.as_str() {
                "machine" => {
                    if i + 1 >= tokens.len() {
                        break;
                    }
                    flush(&mut active, &mut hosts, &mut default);
                    let host = tokens[i + 1].clone();
                    active = Some((Some(host), [None, None, None]));
                    i += 2;
                }
                "default" => {
                    flush(&mut active, &mut hosts, &mut default);
                    active = Some((None, [None, None, None]));
                    i += 1;
                }
                "macdef" => {
                    if i + 1 >= tokens.len() {
                        break;
                    }
                    flush(&mut active, &mut hosts, &mut default);
                    let name = tokens[i + 1].clone();
                    state = State::Macdef(name, Vec::new());
                    // Discard remaining tokens on this line; macdef
                    // body is line-oriented from the next line on.
                    tokens.clear();
                    break;
                }
                "login" | "user" => {
                    if i + 1 >= tokens.len() {
                        break;
                    }
                    if let Some((_, vals)) = active.as_mut() {
                        vals[0] = Some(tokens[i + 1].clone());
                    }
                    i += 2;
                }
                "account" => {
                    if i + 1 >= tokens.len() {
                        break;
                    }
                    if let Some((_, vals)) = active.as_mut() {
                        vals[1] = Some(tokens[i + 1].clone());
                    }
                    i += 2;
                }
                "password" => {
                    if i + 1 >= tokens.len() {
                        break;
                    }
                    if let Some((_, vals)) = active.as_mut() {
                        vals[2] = Some(tokens[i + 1].clone());
                    }
                    i += 2;
                }
                _ => {
                    // Unknown token — skip it (don't terminate the block;
                    // real-world files sometimes have stray whitespace
                    // artifacts).
                    i += 1;
                }
            }
        }
        // Drop consumed tokens; carry-over only happens when we hit
        // `break` in the loop above (= partial keyword pair).
        if matches!(state, State::Top) {
            // If we broke out because of a half-pair like `password`
            // at end-of-line, keep the half-pair so the next line
            // contributes the value.
            if !tokens.is_empty() {
                let last = tokens.last().unwrap().as_str();
                if matches!(
                    last,
                    "machine" | "macdef" | "login" | "user" | "account" | "password" | "default"
                ) {
                    let saved = vec![tokens.pop().unwrap()];
                    tokens.clear();
                    tokens.extend(saved);
                } else {
                    tokens.clear();
                }
            }
        }
    }
    // Tail macdef without trailing blank line.
    if let State::Macdef(name, lines) = state {
        macros.push((name, lines));
    }
    flush(&mut active, &mut hosts, &mut default);

    Parsed {
        hosts,
        default,
        macros,
    }
}

fn build_vals_list(vals: [Option<String>; 3]) -> MbValue {
    let to_v = |opt: Option<String>| -> MbValue {
        match opt {
            Some(s) => MbValue::from_ptr(MbObject::new_str(s)),
            None => MbValue::none(),
        }
    };
    let [a, b, c] = vals;
    MbValue::from_ptr(MbObject::new_list(vec![to_v(a), to_v(b), to_v(c)]))
}

fn parsed_to_dict(p: Parsed) -> MbValue {
    let result = MbObject::new_dict();
    let hosts_dict = MbObject::new_dict();
    let macros_dict = MbObject::new_dict();

    unsafe {
        if let ObjData::Dict(lock) = &(*hosts_dict).data {
            let mut g = lock.write().unwrap();
            for (host, vals) in p.hosts {
                g.insert(
                    super::super::dict_ops::DictKey::Str(host),
                    build_vals_list(vals),
                );
            }
        }
        if let ObjData::Dict(lock) = &(*macros_dict).data {
            let mut g = lock.write().unwrap();
            for (name, lines) in p.macros {
                let line_vs: Vec<MbValue> = lines
                    .into_iter()
                    .map(|l| MbValue::from_ptr(MbObject::new_str(l)))
                    .collect();
                g.insert(
                    super::super::dict_ops::DictKey::Str(name),
                    MbValue::from_ptr(MbObject::new_list(line_vs)),
                );
            }
        }
        if let ObjData::Dict(lock) = &(*result).data {
            let mut g = lock.write().unwrap();
            g.insert(
                super::super::dict_ops::DictKey::Str("hosts".into()),
                MbValue::from_ptr(hosts_dict),
            );
            g.insert(
                super::super::dict_ops::DictKey::Str("macros".into()),
                MbValue::from_ptr(macros_dict),
            );
            if let Some(d) = p.default {
                g.insert(
                    super::super::dict_ops::DictKey::Str("default".into()),
                    build_vals_list(d),
                );
            }
        }
    }
    MbValue::from_ptr(result)
}

unsafe extern "C" fn dispatch_netrc(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let explicit = args.first().copied().and_then(|v| as_str(v));
    let Some(path) = resolve_path(explicit) else {
        return parsed_to_dict(Parsed {
            hosts: vec![],
            default: None,
            macros: vec![],
        });
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => {
            return parsed_to_dict(Parsed {
                hosts: vec![],
                default: None,
                macros: vec![],
            });
        }
    };
    parsed_to_dict(parse_netrc(&text))
}

unsafe extern "C" fn dispatch_netrc_parse_error(_a: *const MbValue, _n: usize) -> MbValue {
    // CPython's NetrcParseError is a real exception subclass; the
    // Mamba runtime doesn't yet promote module-attribute exceptions
    // to first-class types, so callers see a dict shell. They'll
    // detect parse failures by checking that `hosts` is empty.
    MbValue::from_ptr(MbObject::new_dict())
}

pub fn register() {
    let mut attrs = HashMap::new();

    let addr_netrc = dispatch_netrc as *const () as usize;
    let addr_err = dispatch_netrc_parse_error as *const () as usize;

    attrs.insert("netrc".into(), MbValue::from_func(addr_netrc));
    attrs.insert("NetrcParseError".into(), MbValue::from_func(addr_err));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_netrc as u64);
        set.insert(addr_err as u64);
    });

    super::register_module("netrc", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    fn tmp_netrc(name: &str, content: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("mamba_netrc_{}_{}", std::process::id(), name));
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        p
    }

    fn host_vals(parsed: &Parsed, host: &str) -> Option<[Option<String>; 3]> {
        parsed
            .hosts
            .iter()
            .find(|(h, _)| h == host)
            .map(|(_, v)| v.clone())
    }

    #[test]
    fn parses_single_machine_block() {
        let src = "machine example.com\n  login alice\n  password s3cret\n";
        let p = parse_netrc(src);
        let v = host_vals(&p, "example.com").expect("host");
        assert_eq!(v[0].as_deref(), Some("alice"));
        assert_eq!(v[2].as_deref(), Some("s3cret"));
    }

    #[test]
    fn parses_default_block() {
        let src = "default\n  login bob\n  password fallback\n";
        let p = parse_netrc(src);
        let d = p.default.expect("default");
        assert_eq!(d[0].as_deref(), Some("bob"));
        assert_eq!(d[2].as_deref(), Some("fallback"));
    }

    #[test]
    fn parses_two_machines() {
        let src = "machine a.com login u1 password p1\nmachine b.com login u2 password p2\n";
        let p = parse_netrc(src);
        let va = host_vals(&p, "a.com").expect("a.com");
        let vb = host_vals(&p, "b.com").expect("b.com");
        assert_eq!(va[0].as_deref(), Some("u1"));
        assert_eq!(vb[2].as_deref(), Some("p2"));
    }

    #[test]
    fn parses_macdef_body_until_blank_line() {
        let src = "macdef init\n  cd foo\n  ls\n\nmachine x.com login alice password q\n";
        let p = parse_netrc(src);
        let (mname, mlines) = &p.macros[0];
        assert_eq!(mname, "init");
        assert_eq!(mlines.len(), 2);
        assert!(mlines[0].contains("cd foo"));
        let v = host_vals(&p, "x.com").expect("x.com");
        assert_eq!(v[0].as_deref(), Some("alice"));
    }

    #[test]
    fn handles_double_quoted_password() {
        let src = "machine s.com login alice password \"weird pass\"\n";
        let p = parse_netrc(src);
        let v = host_vals(&p, "s.com").expect("host");
        assert_eq!(v[2].as_deref(), Some("weird pass"));
    }

    #[test]
    fn skips_comment_lines() {
        let src = "# comment header\nmachine c.com login u password p\n# trailing\n";
        let p = parse_netrc(src);
        let v = host_vals(&p, "c.com").expect("host");
        assert_eq!(v[0].as_deref(), Some("u"));
    }

    #[test]
    fn parses_account_field() {
        let src = "machine d.com login u password p account team-1\n";
        let p = parse_netrc(src);
        let v = host_vals(&p, "d.com").expect("host");
        assert_eq!(v[1].as_deref(), Some("team-1"));
    }

    #[test]
    fn empty_input_yields_empty_parsed() {
        let p = parse_netrc("");
        assert!(p.hosts.is_empty());
        assert!(p.default.is_none());
        assert!(p.macros.is_empty());
    }

    #[test]
    fn dispatch_reads_file_and_returns_dict() {
        let path = tmp_netrc(
            "dispatch.txt",
            "machine api.example login bot password secret123\n",
        );
        let arg = MbValue::from_ptr(MbObject::new_str(path.to_string_lossy().into_owned()));
        let argv = [arg];
        let result = unsafe { dispatch_netrc(argv.as_ptr(), argv.len()) };
        unsafe {
            let p = result.as_ptr().expect("ptr");
            if let ObjData::Dict(lock) = &(*p).data {
                let g = lock.read().unwrap();
                let hosts_val = g
                    .get(&super::super::super::dict_ops::DictKey::Str("hosts".into()))
                    .expect("hosts");
                let hp = hosts_val.as_ptr().expect("hosts ptr");
                if let ObjData::Dict(hlock) = &(*hp).data {
                    let hg = hlock.read().unwrap();
                    let api_val = hg
                        .get(&super::super::super::dict_ops::DictKey::Str(
                            "api.example".into(),
                        ))
                        .expect("api.example entry");
                    let lp = api_val.as_ptr().expect("list ptr");
                    if let ObjData::List(llock) = &(*lp).data {
                        let items = llock.read().unwrap();
                        let login = items[0]
                            .as_ptr()
                            .and_then(|pp| {
                                if let ObjData::Str(s) = &(*pp).data {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap();
                        assert_eq!(login, "bot");
                    } else {
                        panic!("expected list");
                    }
                } else {
                    panic!("expected hosts dict");
                }
            } else {
                panic!("expected dict");
            }
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_file_returns_empty_dict_not_panic() {
        let arg = MbValue::from_ptr(MbObject::new_str("/does/not/exist/netrc".to_string()));
        let argv = [arg];
        let result = unsafe { dispatch_netrc(argv.as_ptr(), argv.len()) };
        // Just ensure we got a dict back.
        unsafe {
            let p = result.as_ptr().expect("ptr");
            assert!(matches!(&(*p).data, ObjData::Dict(_)));
        }
    }

    #[test]
    fn tokenize_handles_escaped_quote() {
        let toks = tokenize_line("foo \"a \\\"b\" c");
        assert_eq!(toks, vec!["foo", "a \"b", "c"]);
    }
}
