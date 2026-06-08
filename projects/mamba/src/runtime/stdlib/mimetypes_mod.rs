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
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    /// User-added (ext, type) entries via `mimetypes.add_type`.
    /// Consulted before the static table so user overrides win.
    static USER_TYPES: RefCell<Vec<(String, String)>> = const { RefCell::new(Vec::new()) };
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

/// Lookup a mime type for a given suffix. Checks user table first,
/// then the static `TYPES_MAP`. Suffix is the lowercased extension
/// including the leading dot (e.g. `.html`).
fn lookup_type(suffix: &str) -> Option<String> {
    let user = USER_TYPES.with(|t| {
        t.borrow()
            .iter()
            .find(|(ext, _)| ext == suffix)
            .map(|(_, ty)| ty.clone())
    });
    if user.is_some() {
        return user;
    }
    TYPES_MAP
        .iter()
        .find(|(ext, _)| *ext == suffix)
        .map(|(_, ty)| (*ty).to_string())
}

fn lookup_encoding(suffix: &str) -> Option<String> {
    ENCODINGS_MAP
        .iter()
        .find(|(ext, _)| *ext == suffix)
        .map(|(_, enc)| (*enc).to_string())
}

/// Split a url/filename into (base, last_suffix). Returns ("", "")
/// when no `.` is present. Suffix includes the leading dot.
fn split_suffix(url: &str) -> (String, String) {
    if let Some(dot_pos) = url.rfind('.') {
        // Only treat as suffix if the dot is after the last path separator.
        let after_slash = url
            .rfind(|c| c == '/' || c == '\\')
            .map(|p| p + 1)
            .unwrap_or(0);
        if dot_pos >= after_slash {
            return (url[..dot_pos].to_string(), url[dot_pos..].to_string());
        }
    }
    (url.to_string(), String::new())
}

/// guess_type(url, strict=True) -> (type, encoding)
///
/// Resolves the file's suffix chain — `.tar.gz` first hits the
/// encoding map (`.gz` -> `gzip`), then the inner suffix (`.tar`)
/// hits TYPES_MAP. Both result components are returned as a 2-tuple
/// `(Optional[str], Optional[str])`.
pub fn mb_mimetypes_guess_type(url: MbValue, _strict: MbValue) -> MbValue {
    let url_s = extract_str(url).unwrap_or_default();
    let url_lower = url_s.to_lowercase();

    // Walk suffix chain: peel off SUFFIX_MAP aliases first, then
    // peel encoding suffix (.gz/.bz2/...), then look up the residue
    // in TYPES_MAP.
    let mut current = url_lower;
    let mut encoding: Option<String> = None;

    // Apply SUFFIX_MAP alias if present.
    let (base, suffix) = split_suffix(&current);
    if let Some((_, alias)) = SUFFIX_MAP.iter().find(|(ext, _)| *ext == suffix) {
        current = format!("{}{}", base, alias);
    }

    // Peel encoding suffix.
    let (base, suffix) = split_suffix(&current);
    if let Some(enc) = lookup_encoding(&suffix) {
        encoding = Some(enc);
        current = base;
    }

    // Final TYPES_MAP lookup.
    let (_, suffix) = split_suffix(&current);
    let type_str = lookup_type(&suffix);

    let type_val = type_str
        .map(|s| MbValue::from_ptr(MbObject::new_str(s)))
        .unwrap_or_else(MbValue::none);
    let enc_val = encoding
        .map(|s| MbValue::from_ptr(MbObject::new_str(s)))
        .unwrap_or_else(MbValue::none);

    MbValue::from_ptr(MbObject::new_tuple(vec![type_val, enc_val]))
}

/// guess_extension(type, strict=True) -> str | None
///
/// Reverse lookup — first matching suffix for the given mime type.
pub fn mb_mimetypes_guess_extension(type_val: MbValue, _strict: MbValue) -> MbValue {
    let target = extract_str(type_val).unwrap_or_default();
    // User entries override.
    let user_match = USER_TYPES.with(|t| {
        t.borrow()
            .iter()
            .find(|(_, ty)| ty == &target)
            .map(|(ext, _)| ext.clone())
    });
    if let Some(ext) = user_match {
        return MbValue::from_ptr(MbObject::new_str(ext));
    }
    for (ext, ty) in TYPES_MAP {
        if *ty == target {
            return MbValue::from_ptr(MbObject::new_str((*ext).to_string()));
        }
    }
    MbValue::none()
}

/// guess_all_extensions(type, strict=True) -> list[str]
pub fn mb_mimetypes_guess_all_extensions(type_val: MbValue, _strict: MbValue) -> MbValue {
    let target = extract_str(type_val).unwrap_or_default();
    let mut out = Vec::new();
    USER_TYPES.with(|t| {
        for (ext, ty) in t.borrow().iter() {
            if ty == &target {
                out.push(MbValue::from_ptr(MbObject::new_str(ext.clone())));
            }
        }
    });
    for (ext, ty) in TYPES_MAP {
        if *ty == target {
            out.push(MbValue::from_ptr(MbObject::new_str((*ext).to_string())));
        }
    }
    MbValue::from_ptr(MbObject::new_list(out))
}

/// add_type(type, ext, strict=True) -> None
pub fn mb_mimetypes_add_type(type_val: MbValue, ext_val: MbValue, _strict: MbValue) -> MbValue {
    let type_s = extract_str(type_val).unwrap_or_default();
    let ext_s = extract_str(ext_val).unwrap_or_default();
    if !type_s.is_empty() && !ext_s.is_empty() {
        USER_TYPES.with(|t| {
            t.borrow_mut().push((ext_s, type_s));
        });
    }
    MbValue::none()
}

/// init(files=None) -> None  — no-op; the static table is always live.
pub fn mb_mimetypes_init(_files: MbValue) -> MbValue {
    MbValue::none()
}

/// read_mime_types(filename) -> dict | None  — stub returns None.
pub fn mb_mimetypes_read_mime_types(_filename: MbValue) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn dispatch_guess_type(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_mimetypes_guess_type(
        a.first().copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_guess_extension(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_mimetypes_guess_extension(
        a.first().copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_guess_all_extensions(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_mimetypes_guess_all_extensions(
        a.first().copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_add_type(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_mimetypes_add_type(
        a.first().copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        a.get(2).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_init(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_mimetypes_init(a.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_read_mime_types(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_mimetypes_read_mime_types(a.first().copied().unwrap_or_else(MbValue::none))
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

/// Register the mimetypes module.
pub fn register() {
    use super::super::module::NATIVE_FUNC_ADDRS;
    let mut attrs = HashMap::new();

    let dispatchers: &[(&str, usize)] = &[
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
    attrs.insert(
        "common_types".to_string(),
        MbValue::from_ptr(MbObject::new_dict()),
    );

    // MimeTypes class shell (Instance with class_name).
    let class_shell = || -> MbValue {
        use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
        let obj = Box::new(MbObject {
            header: MbObjectHeader {
                rc: std::sync::atomic::AtomicU32::new(1),
                kind: ObjKind::Instance,
            },
            data: ObjData::Instance {
                class_name: "mimetypes.MimeTypes".to_string(),
                fields: crate::runtime::rc::MbRwLock::new(FxHashMap::default()),
            },
        });
        MbValue::from_ptr(Box::into_raw(obj))
    };
    attrs.insert("MimeTypes".to_string(), class_shell());

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
