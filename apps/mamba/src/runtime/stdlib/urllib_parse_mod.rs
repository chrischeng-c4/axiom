//! urllib.parse module for Mamba — Python 3.12 `urllib.parse` stdlib.
//!
//! Provides URL parsing, quote/unquote functions, and module attributes.

use super::super::rc::MbObject;
use super::super::value::MbValue;
use std::collections::HashMap;

pub fn register() {
    let mut attrs = HashMap::new();

    // Export functions from http_mod
    attrs.insert("quote".into(), MbValue::from_func(super::http_mod::dispatch_quote as *const () as usize));
    attrs.insert("quote_plus".into(), MbValue::from_func(super::http_mod::dispatch_quote_plus as *const () as usize));
    attrs.insert("quote_from_bytes".into(), MbValue::from_func(super::http_mod::dispatch_quote_from_bytes as *const () as usize));
    attrs.insert("unquote".into(), MbValue::from_func(super::http_mod::dispatch_unquote as *const () as usize));
    attrs.insert("unquote_plus".into(), MbValue::from_func(super::http_mod::dispatch_unquote_plus as *const () as usize));
    attrs.insert("unquote_to_bytes".into(), MbValue::from_func(super::http_mod::dispatch_unquote_to_bytes as *const () as usize));
    attrs.insert("_unquote_to_bytes".into(), MbValue::from_func(super::http_mod::dispatch_unquote_to_bytes as *const () as usize));
    attrs.insert("urlencode".into(), MbValue::from_func(super::http_mod::dispatch_urlencode as *const () as usize));
    attrs.insert("urlparse".into(), MbValue::from_func(super::http_mod::dispatch_urlparse as *const () as usize));
    attrs.insert("urlunparse".into(), MbValue::from_func(super::http_mod::dispatch_urlunparse as *const () as usize));
    attrs.insert("urlsplit".into(), MbValue::from_func(super::http_mod::dispatch_urlsplit as *const () as usize));
    attrs.insert("urlunsplit".into(), MbValue::from_func(super::http_mod::dispatch_urlunparse as *const () as usize));
    attrs.insert("urljoin".into(), MbValue::from_func(super::http_mod::dispatch_urljoin as *const () as usize));
    attrs.insert("urldefrag".into(), MbValue::from_func(super::http_mod::dispatch_urldefrag as *const () as usize));
    attrs.insert("parse_qs".into(), MbValue::from_func(super::http_mod::dispatch_parse_qs as *const () as usize));
    attrs.insert("parse_qsl".into(), MbValue::from_func(super::http_mod::dispatch_parse_qsl as *const () as usize));
    attrs.insert("unwrap".into(), MbValue::from_func(super::http_mod::dispatch_unwrap as *const () as usize));

    // Register function parameter metadata for quote, unquote, etc.
    let quote_func = MbValue::from_func(super::http_mod::dispatch_quote as *const () as usize);
    let quote_params = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_str("string".to_string())),
            MbValue::from_int(1),
            MbValue::from_int(0),
            MbValue::none(),
            MbValue::none(),
        ])),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_str("safe".to_string())),
            MbValue::from_int(1),
            MbValue::from_int(1),
            MbValue::from_ptr(MbObject::new_str("/".to_string())),
            MbValue::none(),
        ])),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_str("encoding".to_string())),
            MbValue::from_int(1),
            MbValue::from_int(1),
            MbValue::none(),
            MbValue::none(),
        ])),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_str("errors".to_string())),
            MbValue::from_int(1),
            MbValue::from_int(1),
            MbValue::none(),
            MbValue::none(),
        ])),
    ]));
    super::super::closure::mb_func_set_params(quote_func, quote_params);

    let unquote_func = MbValue::from_func(super::http_mod::dispatch_unquote as *const () as usize);
    let unquote_params = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_str("string".to_string())),
            MbValue::from_int(1),
            MbValue::from_int(0),
            MbValue::none(),
            MbValue::none(),
        ])),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_str("encoding".to_string())),
            MbValue::from_int(1),
            MbValue::from_int(1),
            MbValue::from_ptr(MbObject::new_str("utf-8".to_string())),
            MbValue::none(),
        ])),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_str("errors".to_string())),
            MbValue::from_int(1),
            MbValue::from_int(1),
            MbValue::from_ptr(MbObject::new_str("replace".to_string())),
            MbValue::none(),
        ])),
    ]));
    super::super::closure::mb_func_set_params(unquote_func, unquote_params);

    // Module-level constants & dicts
    fn list_of_strs(items: &[&str]) -> MbValue {
        let vals: Vec<MbValue> = items
            .iter()
            .map(|s| MbValue::from_ptr(MbObject::new_str((*s).to_string())))
            .collect();
        MbValue::from_ptr(MbObject::new_list(vals))
    }

    attrs.insert("uses_relative".into(), list_of_strs(&[
        "", "ftp", "http", "gopher", "nntp", "imap", "wais", "file", "https", "shttp", "mms",
        "prospero", "rtsp", "rtspu", "sftp", "svn", "svn+ssh", "ws", "wss",
    ]));
    attrs.insert("uses_netloc".into(), list_of_strs(&[
        "", "ftp", "http", "gopher", "nntp", "telnet", "imap", "wais", "file", "mms",
        "https", "shttp", "snews", "prospero", "rtsp", "rtspu", "rsync", "svn",
        "svn+ssh", "sftp", "nfs", "git", "git+ssh", "ws", "wss", "itms-services",
    ]));
    attrs.insert("uses_params".into(), list_of_strs(&[
        "", "ftp", "hdl", "prospero", "http", "imap", "https", "shttp", "rtsp", "rtspu", "sip",
        "sips", "mms", "sftp", "tel",
    ]));
    attrs.insert("non_hierarchical".into(), list_of_strs(&[
        "gopher", "hdl", "mailto", "news", "telnet", "wais", "imap", "snews", "sip", "sips",
    ]));
    attrs.insert("uses_query".into(), list_of_strs(&[
        "", "http", "wais", "imap", "https", "shttp", "mms", "gopher", "rtsp", "rtspu", "sip",
        "sips",
    ]));
    attrs.insert("uses_fragment".into(), list_of_strs(&[
        "", "ftp", "hdl", "http", "gopher", "news", "nntp", "wais", "https", "shttp", "snews",
        "file", "prospero",
    ]));
    attrs.insert("scheme_chars".into(), MbValue::from_ptr(MbObject::new_str(
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+-.".to_string(),
    )));
    attrs.insert("MAX_CACHE_SIZE".into(), MbValue::from_int(20));
    attrs.insert("_hexdig".into(), MbValue::from_ptr(MbObject::new_str("0123456789ABCDEFabcdef".to_string())));
    attrs.insert("_safe_map".into(), MbValue::from_ptr(MbObject::new_dict()));
    attrs.insert("_fast_quote".into(), MbValue::from_func(super::http_mod::dispatch_quote_from_bytes as *const () as usize));

    // Result types
    let parse_classes: &[&str] = &[
        "ParseResult",
        "ParseResultBytes",
        "SplitResult",
        "SplitResultBytes",
        "DefragResult",
        "DefragResultBytes",
    ];
    for n in parse_classes {
        attrs.insert((*n).into(), MbValue::from_ptr(MbObject::new_str((*n).to_string())));
    }

    super::register_module("urllib.parse", attrs);
}
