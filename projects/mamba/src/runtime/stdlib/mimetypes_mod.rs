//! @codegen-skip: handwrite-pre-standardize
//!
//! mimetypes module for Mamba — Python 3.12 `mimetypes` stdlib (Wave-5 Ship #2, Task #64).
//!
//! Surface:
//!   - guess_type(url, strict=True) -> (type, encoding) 2-tuple
//!   - guess_extension(type, strict=True) -> str | None
//!   - guess_all_extensions(type, strict=True) -> list[str]
//!   - add_type(type, ext, strict=True)
//!   - init(files=None)  -- no-op; built-in table is always initialized
//!   - read_mime_types(filename) -> dict | None (no-op stub)
//!   - inited (bool module attr, always True)
//!   - common_types (dict[ext, type] -- empty for forward ship)
//!   - encodings_map, suffix_map, types_map (dict module attrs)
//!   - MimeTypes class shell (Instance with class_name)
//!
//! Implementation: static baked-at-compile-time `(suffix, type)` table
//! mirrors CPython's Lib/mimetypes.py types_map_default. User-added
//! types via `add_type` go into a thread-local `Vec<(String, String)>`
//! consulted before the static table.
//!
//! Subset B avoidance: `guess_type` returns a 2-tuple per call but the
//! tuple contents are short interned strs — per-call alloc cost is
//! ~2 small str allocs + 1 tuple, dominated by hash-table lookup.
//! Predicted Gate 2: wall >=3.0x PASS (scout estimate), internal
//! borderline PASS (no Instance allocation), mem ~1.0x PASS.

use super::super::rc::MbObject;
use super::super::value::MbValue;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    /// User-added `(ext, type, strict)` entries via `mimetypes.add_type`.
    /// Consulted before the static table so user overrides win. The `strict`
    /// flag mirrors CPython's two-table routing: `add_type(..., strict=True)`
    /// (the default) lands in the strict-visible table, `strict=False` in the
    /// loose-only table that only `strict=False` lookups consult.
    // The type is stored as an MbValue (not String) so add_type can preserve a
    // non-str type object (CPython allows it) and guess_type returns it verbatim.
    // ptr-typed values are retain'd on insert and release'd on clear.
    static USER_TYPES: RefCell<Vec<(String, MbValue, bool)>> = const { RefCell::new(Vec::new()) };
}

/// Compile-time baked (suffix, mime-type) table — mirrors CPython 3.12
/// `mimetypes.MimeTypes.__init__` default registration order
/// (insertion order is significant: `guess_extension` returns the
/// first matching suffix for a given type, so the order here is the
/// observable order for reverse lookups). 152 entries.
const TYPES_MAP: &[(&str, &str)] = &[
    (".js", "text/javascript"),
    (".mjs", "text/javascript"),
    (".json", "application/json"),
    (".webmanifest", "application/manifest+json"),
    (".doc", "application/msword"),
    (".dot", "application/msword"),
    (".wiz", "application/msword"),
    (".nq", "application/n-quads"),
    (".nt", "application/n-triples"),
    (".bin", "application/octet-stream"),
    (".a", "application/octet-stream"),
    (".dll", "application/octet-stream"),
    (".exe", "application/octet-stream"),
    (".o", "application/octet-stream"),
    (".obj", "application/octet-stream"),
    (".so", "application/octet-stream"),
    (".oda", "application/oda"),
    (".pdf", "application/pdf"),
    (".p7c", "application/pkcs7-mime"),
    (".ps", "application/postscript"),
    (".ai", "application/postscript"),
    (".eps", "application/postscript"),
    (".trig", "application/trig"),
    (".m3u", "application/vnd.apple.mpegurl"),
    (".m3u8", "application/vnd.apple.mpegurl"),
    (".xls", "application/vnd.ms-excel"),
    (".xlb", "application/vnd.ms-excel"),
    (".ppt", "application/vnd.ms-powerpoint"),
    (".pot", "application/vnd.ms-powerpoint"),
    (".ppa", "application/vnd.ms-powerpoint"),
    (".pps", "application/vnd.ms-powerpoint"),
    (".pwz", "application/vnd.ms-powerpoint"),
    (".wasm", "application/wasm"),
    (".bcpio", "application/x-bcpio"),
    (".cpio", "application/x-cpio"),
    (".csh", "application/x-csh"),
    (".dvi", "application/x-dvi"),
    (".gtar", "application/x-gtar"),
    (".hdf", "application/x-hdf"),
    (".h5", "application/x-hdf5"),
    (".latex", "application/x-latex"),
    (".mif", "application/x-mif"),
    (".cdf", "application/x-netcdf"),
    (".nc", "application/x-netcdf"),
    (".p12", "application/x-pkcs12"),
    (".pfx", "application/x-pkcs12"),
    (".ram", "application/x-pn-realaudio"),
    (".pyc", "application/x-python-code"),
    (".pyo", "application/x-python-code"),
    (".sh", "application/x-sh"),
    (".shar", "application/x-shar"),
    (".swf", "application/x-shockwave-flash"),
    (".sv4cpio", "application/x-sv4cpio"),
    (".sv4crc", "application/x-sv4crc"),
    (".tar", "application/x-tar"),
    (".tcl", "application/x-tcl"),
    (".tex", "application/x-tex"),
    (".texi", "application/x-texinfo"),
    (".texinfo", "application/x-texinfo"),
    (".roff", "application/x-troff"),
    (".t", "application/x-troff"),
    (".tr", "application/x-troff"),
    (".man", "application/x-troff-man"),
    (".me", "application/x-troff-me"),
    (".ms", "application/x-troff-ms"),
    (".ustar", "application/x-ustar"),
    (".src", "application/x-wais-source"),
    (".xsl", "application/xml"),
    (".rdf", "application/xml"),
    (".wsdl", "application/xml"),
    (".xpdl", "application/xml"),
    (".zip", "application/zip"),
    (".3gp", "audio/3gpp"),
    (".3gpp", "audio/3gpp"),
    (".3g2", "audio/3gpp2"),
    (".3gpp2", "audio/3gpp2"),
    (".aac", "audio/aac"),
    (".adts", "audio/aac"),
    (".loas", "audio/aac"),
    (".ass", "audio/aac"),
    (".au", "audio/basic"),
    (".snd", "audio/basic"),
    (".mp3", "audio/mpeg"),
    (".mp2", "audio/mpeg"),
    (".opus", "audio/opus"),
    (".aif", "audio/x-aiff"),
    (".aifc", "audio/x-aiff"),
    (".aiff", "audio/x-aiff"),
    (".ra", "audio/x-pn-realaudio"),
    (".wav", "audio/x-wav"),
    (".avif", "image/avif"),
    (".bmp", "image/bmp"),
    (".gif", "image/gif"),
    (".ief", "image/ief"),
    (".jpg", "image/jpeg"),
    (".jpe", "image/jpeg"),
    (".jpeg", "image/jpeg"),
    (".heic", "image/heic"),
    (".heif", "image/heif"),
    (".png", "image/png"),
    (".svg", "image/svg+xml"),
    (".tiff", "image/tiff"),
    (".tif", "image/tiff"),
    (".ico", "image/vnd.microsoft.icon"),
    (".ras", "image/x-cmu-raster"),
    (".pnm", "image/x-portable-anymap"),
    (".pbm", "image/x-portable-bitmap"),
    (".pgm", "image/x-portable-graymap"),
    (".ppm", "image/x-portable-pixmap"),
    (".rgb", "image/x-rgb"),
    (".xbm", "image/x-xbitmap"),
    (".xpm", "image/x-xpixmap"),
    (".xwd", "image/x-xwindowdump"),
    (".eml", "message/rfc822"),
    (".mht", "message/rfc822"),
    (".mhtml", "message/rfc822"),
    (".nws", "message/rfc822"),
    (".css", "text/css"),
    (".csv", "text/csv"),
    (".html", "text/html"),
    (".htm", "text/html"),
    (".md", "text/markdown"),
    (".markdown", "text/markdown"),
    (".n3", "text/n3"),
    (".txt", "text/plain"),
    (".bat", "text/plain"),
    (".c", "text/plain"),
    (".h", "text/plain"),
    (".ksh", "text/plain"),
    (".pl", "text/plain"),
    (".srt", "text/plain"),
    (".rtx", "text/richtext"),
    (".tsv", "text/tab-separated-values"),
    (".vtt", "text/vtt"),
    (".py", "text/x-python"),
    (".rst", "text/x-rst"),
    (".etx", "text/x-setext"),
    (".sgm", "text/x-sgml"),
    (".sgml", "text/x-sgml"),
    (".vcf", "text/x-vcard"),
    (".xml", "text/xml"),
    (".mp4", "video/mp4"),
    (".mpeg", "video/mpeg"),
    (".m1v", "video/mpeg"),
    (".mpa", "video/mpeg"),
    (".mpe", "video/mpeg"),
    (".mpg", "video/mpeg"),
    (".mov", "video/quicktime"),
    (".qt", "video/quicktime"),
    (".webm", "video/webm"),
    (".avi", "video/x-msvideo"),
    (".movie", "video/x-sgi-movie"),
];

/// Compile-time baked encodings map (suffix -> encoding for .gz/.bz2/etc.).
const ENCODINGS_MAP: &[(&str, &str)] = &[
    (".gz", "gzip"),
    (".Z", "compress"),
    (".bz2", "bzip2"),
    (".xz", "xz"),
    (".br", "br"),
];

/// Suffixes that pass through to the underlying type before encoding
/// is applied (e.g. `.tar.gz` -> `(application/x-tar, gzip)`).
const SUFFIX_MAP: &[(&str, &str)] = &[
    (".svgz", ".svg.gz"),
    (".tgz", ".tar.gz"),
    (".taz", ".tar.gz"),
    (".tz", ".tar.gz"),
    (".tbz2", ".tar.bz2"),
    (".txz", ".tar.xz"),
];

/// Non-standard but commonly found `(suffix, type)` entries that are only
/// consulted when `strict=False`. Mirrors CPython 3.12
/// `mimetypes._common_types` insertion order (significant for reverse
/// `guess_extension` lookups).
const COMMON_TYPES: &[(&str, &str)] = &[
    (".rtf", "application/rtf"),
    (".midi", "audio/midi"),
    (".mid", "audio/midi"),
    (".jpg", "image/jpg"),
    (".pict", "image/pict"),
    (".pct", "image/pict"),
    (".pic", "image/pict"),
    (".webp", "image/webp"),
    (".xul", "text/xul"),
];

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Decode a path-like argument the way CPython's `guess_type` does via
/// `os.fspath`: a `str` passes through; an instance implementing the
/// `__fspath__` protocol (any `os.PathLike`) is decoded by calling it. Returns
/// `None` for anything else (mirroring mamba's non-path arg handling).
fn extract_pathlike(val: MbValue) -> Option<String> {
    if let Some(s) = extract_str(val) {
        return Some(s);
    }
    let ptr = val.as_ptr()?;
    use super::super::rc::ObjData;
    let is_instance = unsafe { matches!((*ptr).data, ObjData::Instance { .. }) };
    if !is_instance {
        return None;
    }
    let has = unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            !super::super::class::lookup_method(class_name, "__fspath__").is_none()
        } else {
            false
        }
    };
    if !has {
        return None;
    }
    let method = MbValue::from_ptr(MbObject::new_str("__fspath__".to_string()));
    let empty = MbValue::from_ptr(MbObject::new_list(Vec::new()));
    let result = super::super::class::mb_call_method(val, method, empty);
    extract_str(result)
}

/// True when `val` is a dict carrying at least one of mimetypes' public
/// keyword-argument names — the trailing kwargs dict that HIR lowering appends
/// for `f(url=..., strict=...)` / `f(type=..., strict=...)` calls. A plain
/// string/bool positional arg is never mistaken for it.
fn is_kwargs_dict(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe {
            use super::super::rc::ObjData;
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                map.get("url").is_some() || map.get("type").is_some() || map.get("strict").is_some()
            } else {
                false
            }
        })
        .unwrap_or(false)
}

/// Read a named keyword value out of a trailing kwargs dict, if present.
fn kwarg(val: MbValue, key: &str) -> Option<MbValue> {
    val.as_ptr().and_then(|ptr| unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

/// Resolve the positional-or-keyword `(value, strict)` argument pair for the
/// `guess_*`/`add_type` surface from the raw dispatcher slice. `args` is the
/// flat call slice (receiver excluded). Keyword calls arrive as a trailing
/// kwargs dict appended by HIR lowering; the value parameter is named `value_kw`
/// (`url`/`type`) and `strict` is always `strict`.
fn resolve_value_strict(args: &[MbValue], value_kw: &str) -> (MbValue, bool) {
    // Locate a trailing kwargs dict, if any.
    let kw = args.iter().copied().find(|v| is_kwargs_dict(*v));
    // The leading value is the first positional that is not the kwargs dict,
    // else the keyword form.
    let value = args
        .iter()
        .copied()
        .find(|v| !is_kwargs_dict(*v))
        .or_else(|| kw.and_then(|k| kwarg(k, value_kw)))
        .unwrap_or_else(MbValue::none);
    // strict: a positional bool after the value, or the `strict` keyword.
    let strict_val = kw
        .and_then(|k| kwarg(k, "strict"))
        .or_else(|| {
            // Second non-kwargs positional, if present (e.g. guess_type(u, False)).
            args.iter().copied().filter(|v| !is_kwargs_dict(*v)).nth(1)
        })
        .unwrap_or_else(MbValue::none);
    let strict_flag = !matches!(strict_val.as_bool(), Some(false));
    (value, strict_flag)
}

/// Lookup a mime type for a given suffix. Checks user table first,
/// then the static `TYPES_MAP`, and finally (when `strict` is false) the
/// non-standard `COMMON_TYPES` table. Suffix is the lowercased extension
/// including the leading dot (e.g. `.html`).
fn lookup_type(suffix: &str, strict: bool) -> Option<MbValue> {
    let user = USER_TYPES.with(|t| {
        t.borrow()
            .iter()
            // A loose-only (`strict=False`) user entry is invisible to a strict
            // lookup; strict entries are visible to both.
            .find(|(ext, _, entry_strict)| ext == suffix && (*entry_strict || !strict))
            .map(|(_, ty, _)| {
                unsafe { super::super::rc::retain_if_ptr(*ty); }
                *ty
            })
    });
    if user.is_some() {
        return user;
    }
    if let Some(ty) = TYPES_MAP.iter().find(|(ext, _)| *ext == suffix) {
        return Some(MbValue::from_ptr(MbObject::new_str(ty.1.to_string())));
    }
    if !strict {
        if let Some(ty) = COMMON_TYPES.iter().find(|(ext, _)| *ext == suffix) {
            return Some(MbValue::from_ptr(MbObject::new_str(ty.1.to_string())));
        }
    }
    None
}

fn lookup_encoding(suffix: &str) -> Option<String> {
    ENCODINGS_MAP
        .iter()
        .find(|(ext, _)| *ext == suffix)
        .map(|(_, enc)| (*enc).to_string())
}

/// `posixpath.splitext` port: split `path` into `(root, ext)` where `ext`
/// is the final extension including the leading dot, but only when there is
/// at least one non-leading-dot character before it (so `.cshrc` and `..`
/// have no extension). The split is computed on the last path component:
/// a dot in a directory segment never produces an extension.
fn posix_splitext(path: &str) -> (String, String) {
    let sep_idx = path.rfind('/').map(|p| p as isize).unwrap_or(-1);
    // Find the last dot at-or-after the basename start.
    let dot_idx = match path.rfind('.') {
        Some(d) if (d as isize) > sep_idx => d,
        _ => return (path.to_string(), String::new()),
    };
    // Skip leading dots in the basename (e.g. "...ext" / ".cshrc").
    let mut file_start = (sep_idx + 1) as usize;
    while file_start < dot_idx && path.as_bytes()[file_start] == b'.' {
        file_start += 1;
    }
    if file_start >= dot_idx {
        // The dot is a leading dot — no extension.
        return (path.to_string(), String::new());
    }
    (path[..dot_idx].to_string(), path[dot_idx..].to_string())
}

/// Port of `urllib.parse.urlparse` scheme/path extraction as used by
/// `mimetypes.guess_type`. Returns `(scheme, path)` where `scheme` is the
/// URL scheme in lowercase (or empty) and `path` is the path component with
/// any `?query` / `#fragment` stripped off when a scheme is present.
///
/// CPython only treats the leading token as a scheme when it starts with an
/// ASCII letter, is followed by `:`, and contains only `[A-Za-z0-9+.-]`.
fn url_scheme_path(url: &str) -> (String, String) {
    if let Some(colon) = url.find(':') {
        let head = &url[..colon];
        let valid = !head.is_empty()
            && head.as_bytes()[0].is_ascii_alphabetic()
            && head
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'-' || b == b'.');
        // CPython's guess_type requires len(scheme) > 1 to treat it as a URL
        // scheme (so a Windows drive letter "c:" stays a path).
        if valid && head.len() > 1 {
            let scheme = head.to_ascii_lowercase();
            let rest = &url[colon + 1..];
            // Strip authority for `//host...` forms, then drop query/fragment.
            let after_authority = if let Some(stripped) = rest.strip_prefix("//") {
                match stripped.find(['/', '?', '#']) {
                    Some(p) => &stripped[p..],
                    None => "",
                }
            } else {
                rest
            };
            let path = after_authority
                .split(['?', '#'])
                .next()
                .unwrap_or("")
                .to_string();
            return (scheme, path);
        }
    }
    (String::new(), url.to_string())
}

/// guess_type(url, strict=True) -> (type, encoding)
///
/// Faithful port of CPython 3.12 `mimetypes.MimeTypes.guess_type`:
///   1. Split the URL scheme; for non-`data` URLs the path component (with
///      `?query`/`#fragment` stripped) is used.
///   2. `data:` URLs are parsed for their inline media type.
///   3. Suffix-map aliases (`.tgz` -> `.tar.gz`) are applied case-INsensitively.
///   4. The encoding map (`.gz` -> gzip) is consulted case-SENSITIVELY.
///   5. The final extension is lowercased before the types-map lookup.
pub fn mb_mimetypes_guess_type(url: MbValue, strict: MbValue) -> MbValue {
    // guess_type requires a str/bytes/PathLike; a bare scalar (int/None/float)
    // is a TypeError (os.fspath rejects it), not a silent ("", None) result.
    if url.as_ptr().is_none() {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "expected str, bytes or os.PathLike object, not {}",
                super::super::builtins::value_type_name(url)
            ))),
        );
        return MbValue::none();
    }
    let url_s = extract_pathlike(url).unwrap_or_default();
    let strict_flag = !matches!(strict.as_bool(), Some(false));
    let none_pair =
        || MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::none(), MbValue::none()]));

    let (scheme, mut current) = url_scheme_path(&url_s);

    // data: URL — extract the inline media type.
    if scheme == "data" {
        let comma = match current.find(',') {
            Some(c) => c,
            None => return none_pair(), // bad data URL
        };
        let before = &current[..comma];
        let mut ty = match before.find(';') {
            Some(semi) => before[..semi].to_string(),
            None => before.to_string(),
        };
        if ty.contains('=') || !ty.contains('/') {
            ty = "text/plain".to_string();
        }
        return MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_str(ty)),
            MbValue::none(),
        ]));
    }

    // Suffix-map aliases are matched case-insensitively (ext.lower()).
    let (mut base, mut ext) = posix_splitext(&current);
    loop {
        let ext_lower = ext.to_lowercase();
        match SUFFIX_MAP.iter().find(|(k, _)| *k == ext_lower) {
            Some((_, alias)) => {
                current = format!("{}{}", base, alias);
                let (b, e) = posix_splitext(&current);
                base = b;
                ext = e;
            }
            None => break,
        }
    }

    // Encoding map is CASE-SENSITIVE (ext used verbatim).
    let mut encoding: Option<String> = None;
    if let Some(enc) = lookup_encoding(&ext) {
        encoding = Some(enc);
        let (_, e) = posix_splitext(&base);
        ext = e;
    }

    // Types map: lowercase the final extension before lookup.
    let ext_lower = ext.to_lowercase();
    let type_val = if ext_lower.is_empty() {
        MbValue::none()
    } else {
        lookup_type(&ext_lower, strict_flag).unwrap_or_else(MbValue::none)
    };
    let enc_val = encoding
        .map(|s| MbValue::from_ptr(MbObject::new_str(s)))
        .unwrap_or_else(MbValue::none);

    MbValue::from_ptr(MbObject::new_tuple(vec![type_val, enc_val]))
}

/// Collect all extensions registered for `target` (a lowercased mime type),
/// in CPython 3.12 `types_map_inv` order: user-added entries (in add order),
/// then the static strict table, then — only when `strict` is false — the
/// non-standard common table. Duplicates are suppressed.
fn collect_extensions(target: &str, strict: bool) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut push = |ext: String| {
        if !out.contains(&ext) {
            out.push(ext);
        }
    };
    USER_TYPES.with(|t| {
        for (ext, ty, entry_strict) in t.borrow().iter() {
            // Loose-only user entries surface only under a `strict=False` query.
            if extract_str(*ty).as_deref() == Some(target) && (*entry_strict || !strict) {
                push(ext.clone());
            }
        }
    });
    for (ext, ty) in TYPES_MAP {
        if *ty == target {
            push((*ext).to_string());
        }
    }
    if !strict {
        for (ext, ty) in COMMON_TYPES {
            if *ty == target {
                push((*ext).to_string());
            }
        }
    }
    out
}

/// guess_extension(type, strict=True) -> str | None
///
/// Reverse lookup — first matching suffix for the given mime type.
pub fn mb_mimetypes_guess_extension(type_val: MbValue, strict: MbValue) -> MbValue {
    let target = extract_str(type_val).unwrap_or_default().to_lowercase();
    let strict_flag = !matches!(strict.as_bool(), Some(false));
    match collect_extensions(&target, strict_flag).into_iter().next() {
        Some(ext) => MbValue::from_ptr(MbObject::new_str(ext)),
        None => MbValue::none(),
    }
}

/// guess_all_extensions(type, strict=True) -> list[str]
pub fn mb_mimetypes_guess_all_extensions(type_val: MbValue, strict: MbValue) -> MbValue {
    let target = extract_str(type_val).unwrap_or_default().to_lowercase();
    let strict_flag = !matches!(strict.as_bool(), Some(false));
    let out = collect_extensions(&target, strict_flag)
        .into_iter()
        .map(|ext| MbValue::from_ptr(MbObject::new_str(ext)))
        .collect();
    MbValue::from_ptr(MbObject::new_list(out))
}

/// add_type(type, ext, strict=True) -> None
///
/// `strict` (default True) routes the entry into the strict-visible table; a
/// `strict=False` registration lands in the loose-only table so it surfaces
/// only under `strict=False` lookups (mirrors CPython's two-table model).
pub fn mb_mimetypes_add_type(type_val: MbValue, ext_val: MbValue, strict: MbValue) -> MbValue {
    let ext_s = extract_str(ext_val).unwrap_or_default();
    let strict_flag = !matches!(strict.as_bool(), Some(false));
    // CPython accepts any type object (not just str); store it verbatim. An
    // empty extension or a None type is a no-op.
    if !ext_s.is_empty() && !type_val.is_none() {
        unsafe { super::super::rc::retain_if_ptr(type_val); }
        USER_TYPES.with(|t| {
            t.borrow_mut().push((ext_s, type_val, strict_flag));
        });
    }
    MbValue::none()
}

/// Instance-method bindings for the `MimeTypes` dict stub: `(name, addr)` of
/// the native dispatcher to seed into a fresh instance so `db.<name>(...)`
/// resolves to the same code path as the module-level `mimetypes.<name>(...)`.
fn instance_methods() -> [(&'static str, usize); 6] {
    [
        ("guess_type", dispatch_guess_type as *const () as usize),
        (
            "guess_extension",
            dispatch_guess_extension as *const () as usize,
        ),
        (
            "guess_all_extensions",
            dispatch_guess_all_extensions as *const () as usize,
        ),
        ("add_type", dispatch_add_type as *const () as usize),
        (
            "read_mime_types",
            dispatch_read_mime_types as *const () as usize,
        ),
        ("read", dispatch_read as *const () as usize),
    ]
}

/// MimeTypes(filenames=(), strict=True) -> instance
///
/// Constructs a `mimetypes.MimeTypes` database object. Modeled as a Dict
/// stub carrying `__class__: "MimeTypes"`. Instance-method dispatch
/// (`db.guess_type(...)`) is wired by seeding the instance dict with the
/// module's native dispatcher func-values under their method names: the core
/// Dict method-call path (`class.rs`) resolves a same-named callable dict entry
/// before falling back to the generic dict-method table, calling it with the
/// `(args_ptr, nargs)` native ABI (receiver excluded) — identical to the
/// module-level `mimetypes.guess_type(...)` call shape.
pub fn mb_mimetypes_MimeTypes() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut m = lock.write().unwrap();
            m.insert(
                "__class__".into(),
                MbValue::from_ptr(MbObject::new_str("MimeTypes".to_string())),
            );
            // Bind the bound-method surface so `db.<method>(...)` dispatches to
            // the same native dispatchers the module-level functions use. The
            // addresses are registered in `NATIVE_FUNC_ADDRS` by `register()`,
            // so `is_native_func` recognises them and applies the flat-args ABI.
            for (name, addr) in instance_methods() {
                m.insert(name.into(), MbValue::from_func(addr));
            }
        }
    }
    MbValue::from_ptr(dict)
}

/// init(files=None) -> None — reset the module-level registry maps.
pub fn mb_mimetypes_init(_files: MbValue) -> MbValue {
    // CPython's init() rebuilds the registry from the system mime files; mamba
    // has no system files, so reset to the static defaults by clearing the
    // user-added types (so an add_type() before init() is forgotten) and by
    // replacing the public maps with fresh dict objects.
    USER_TYPES.with(|t| {
        let mut v = t.borrow_mut();
        for (_, ty, _) in v.iter() {
            unsafe { super::super::rc::release_if_ptr(*ty); }
        }
        v.clear();
    });
    set_module_map("types_map", build_static_dict(TYPES_MAP));
    set_module_map("encodings_map", build_static_dict(ENCODINGS_MAP));
    set_module_map("suffix_map", build_static_dict(SUFFIX_MAP));
    set_module_map("common_types", build_static_dict(COMMON_TYPES));
    MbValue::none()
}

/// read_mime_types(filename) -> dict | None
///
/// Port of CPython 3.12 `mimetypes.read_mime_types`: open `filename`, parse
/// each `type ext ext...` line (truncating at the first `#`-prefixed word),
/// and return the resulting `{'.ext': 'type'}` mapping seeded with the
/// default `types_map`. Returns `None` if the file cannot be opened.
pub fn mb_mimetypes_read_mime_types(filename: MbValue) -> MbValue {
    let path = match extract_str(filename) {
        Some(p) => p,
        None => return MbValue::none(),
    };
    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return MbValue::none(),
    };

    // Seed with the strict default table (CPython returns db.types_map[True],
    // which already contains every default entry).
    let dict = MbObject::new_dict();
    unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            for (ext, ty) in TYPES_MAP {
                map.insert(
                    (*ext).into(),
                    MbValue::from_ptr(MbObject::new_str((*ty).to_string())),
                );
            }
            for raw in contents.lines() {
                // Split on whitespace, then truncate at the first word that
                // begins with '#' (an inline comment).
                let mut words: Vec<&str> = raw.split_whitespace().collect();
                if let Some(pos) = words.iter().position(|w| w.starts_with('#')) {
                    words.truncate(pos);
                }
                if words.is_empty() {
                    continue;
                }
                let ty = words[0];
                for suff in &words[1..] {
                    let key = format!(".{}", suff);
                    map.insert(
                        key.into(),
                        MbValue::from_ptr(MbObject::new_str(ty.to_string())),
                    );
                }
            }
        }
    }
    MbValue::from_ptr(dict)
}

unsafe extern "C" fn dispatch_MimeTypes(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_mimetypes_MimeTypes()
}

unsafe extern "C" fn dispatch_guess_type(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (url, strict_flag) = resolve_value_strict(a, "url");
    mb_mimetypes_guess_type(url, MbValue::from_bool(strict_flag))
}

unsafe extern "C" fn dispatch_guess_extension(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (type_v, strict_flag) = resolve_value_strict(a, "type");
    mb_mimetypes_guess_extension(type_v, MbValue::from_bool(strict_flag))
}

unsafe extern "C" fn dispatch_guess_all_extensions(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (type_v, strict_flag) = resolve_value_strict(a, "type");
    mb_mimetypes_guess_all_extensions(type_v, MbValue::from_bool(strict_flag))
}

unsafe extern "C" fn dispatch_add_type(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // `type` and `ext` are the first two non-kwargs positionals; `strict` is a
    // positional bool after them or the `strict=` keyword in the trailing dict.
    let mut positional = a.iter().copied().filter(|v| !is_kwargs_dict(*v));
    let type_v = positional.next().unwrap_or_else(MbValue::none);
    let ext_v = positional.next().unwrap_or_else(MbValue::none);
    let strict_v = positional
        .next()
        .or_else(|| {
            a.iter()
                .copied()
                .find(|v| is_kwargs_dict(*v))
                .and_then(|k| kwarg(k, "strict"))
        })
        .unwrap_or_else(MbValue::none);
    mb_mimetypes_add_type(type_v, ext_v, strict_v)
}

unsafe extern "C" fn dispatch_init(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_mimetypes_init(a.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_read_mime_types(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_mimetypes_read_mime_types(a.first().copied().unwrap_or_else(MbValue::none))
}

/// `MimeTypes().read(filename)` — unlike the module-level `read_mime_types`
/// (which catches and returns None), the instance method opens the file and so
/// a missing path raises FileNotFoundError (CPython).
unsafe extern "C" fn dispatch_read(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let filename = a.first().copied().unwrap_or_else(MbValue::none);
    let path = extract_str(filename).unwrap_or_default();
    if std::fs::metadata(&path).is_err() {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("FileNotFoundError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "[Errno 2] No such file or directory: '{path}'"
            ))),
        );
        return MbValue::none();
    }
    mb_mimetypes_read_mime_types(filename)
}

/// Build a Dict MbValue from a static (key, val) slice.
fn build_static_dict(entries: &[(&str, &str)]) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            for (k, v) in entries {
                map.insert(
                    (*k).into(),
                    MbValue::from_ptr(MbObject::new_str((*v).to_string())),
                );
            }
        }
    }
    MbValue::from_ptr(dict)
}

fn str_value(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn set_module_map(attr: &str, value: MbValue) {
    super::super::module::mb_module_setattr(str_value("mimetypes"), str_value(attr), value);
    let cached = super::super::module::MODULES.with(|mods| {
        mods.borrow().get("mimetypes").and_then(|module| module.cached_value)
    });
    if let Some(module_value) = cached {
        super::super::dict_ops::mb_dict_setitem(module_value, str_value(attr), value);
    }
}

/// Register the mimetypes module.
pub fn register() {
    use super::super::module::NATIVE_FUNC_ADDRS;
    let mut attrs = HashMap::new();

    let dispatchers: &[(&str, usize)] = &[
        ("MimeTypes", dispatch_MimeTypes as *const () as usize),
        ("guess_type", dispatch_guess_type as *const () as usize),
        (
            "guess_extension",
            dispatch_guess_extension as *const () as usize,
        ),
        (
            "guess_all_extensions",
            dispatch_guess_all_extensions as *const () as usize,
        ),
        ("add_type", dispatch_add_type as *const () as usize),
        ("init", dispatch_init as *const () as usize),
        (
            "read_mime_types",
            dispatch_read_mime_types as *const () as usize,
        ),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).to_string(), MbValue::from_func(*addr));
        NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
    }
    // `read` is an instance-only method (not a module-level function), so it is
    // not in the dispatchers list above — register its addr so is_native_func
    // recognises it when dispatched off a MimeTypes instance.
    NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut()
            .insert(dispatch_read as *const () as usize as u64);
    });

    // Module attrs.
    attrs.insert("inited".to_string(), MbValue::from_bool(true));
    attrs.insert(
        "knownfiles".to_string(),
        MbValue::from_ptr(MbObject::new_list(vec![])),
    );

    // Static maps as Dict module attrs.
    attrs.insert("types_map".to_string(), build_static_dict(TYPES_MAP));
    attrs.insert(
        "encodings_map".to_string(),
        build_static_dict(ENCODINGS_MAP),
    );
    attrs.insert("suffix_map".to_string(), build_static_dict(SUFFIX_MAP));
    attrs.insert("common_types".to_string(), build_static_dict(COMMON_TYPES));

    // `MimeTypes` is registered above as a callable constructor (native func)
    // so that `mimetypes.MimeTypes()` works like `configparser.ConfigParser()`.
    // Instance-method dispatch (`db.guess_type(...)`) is wired by seeding each
    // fresh instance dict with the same native dispatcher func-values (see
    // `instance_methods`); the core Dict method-call path in `class.rs` resolves
    // those same-named callable entries with the flat-args ABI.

    super::register_module("mimetypes", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_str(s: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(s.to_string()))
    }

    fn tuple_first_str(t: MbValue) -> Option<String> {
        use super::super::super::rc::ObjData;
        t.as_ptr().and_then(|p| unsafe {
            if let ObjData::Tuple(ref items) = (*p).data {
                items.first().and_then(|v| extract_str(*v))
            } else {
                None
            }
        })
    }

    fn tuple_second_str(t: MbValue) -> Option<String> {
        use super::super::super::rc::ObjData;
        t.as_ptr().and_then(|p| unsafe {
            if let ObjData::Tuple(ref items) = (*p).data {
                items.get(1).and_then(|v| extract_str(*v))
            } else {
                None
            }
        })
    }

    #[test]
    fn test_guess_type_html() {
        let r = mb_mimetypes_guess_type(make_str("page.html"), MbValue::none());
        assert_eq!(tuple_first_str(r).as_deref(), Some("text/html"));
        assert_eq!(tuple_second_str(r), None);
    }

    #[test]
    fn test_guess_type_tar_gz() {
        let r = mb_mimetypes_guess_type(make_str("backup.tar.gz"), MbValue::none());
        assert_eq!(tuple_first_str(r).as_deref(), Some("application/x-tar"));
        assert_eq!(tuple_second_str(r).as_deref(), Some("gzip"));
    }

    #[test]
    fn test_guess_type_unknown() {
        let r = mb_mimetypes_guess_type(make_str("file.zzz"), MbValue::none());
        assert_eq!(tuple_first_str(r), None);
        assert_eq!(tuple_second_str(r), None);
    }

    #[test]
    fn test_guess_extension_jpeg() {
        // CPython 3.12 insertion order: .jpg first.
        let r = mb_mimetypes_guess_extension(make_str("image/jpeg"), MbValue::none());
        let ext = extract_str(r);
        assert_eq!(ext.as_deref(), Some(".jpg"));
    }

    #[test]
    fn test_guess_type_js_is_text_javascript() {
        // CPython 3.12 changed .js from application/javascript to text/javascript.
        let r = mb_mimetypes_guess_type(make_str("app.js"), MbValue::none());
        assert_eq!(tuple_first_str(r).as_deref(), Some("text/javascript"));
    }

    #[test]
    fn test_guess_extension_html_first() {
        // CPython 3.12 lists .html before .htm.
        let r = mb_mimetypes_guess_extension(make_str("text/html"), MbValue::none());
        assert_eq!(extract_str(r).as_deref(), Some(".html"));
    }

    #[test]
    fn test_add_type_overrides() {
        mb_mimetypes_add_type(
            make_str("application/x-custom"),
            make_str(".cstm"),
            MbValue::none(),
        );
        let r = mb_mimetypes_guess_type(make_str("file.cstm"), MbValue::none());
        assert_eq!(tuple_first_str(r).as_deref(), Some("application/x-custom"));
    }
}
