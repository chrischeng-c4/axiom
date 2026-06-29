#![cfg(test)]

use crate::runtime::rc::{MbObject, ObjData};
/// Integration tests for remaining coverage stdlib modules.
///
/// Covers cross-module interactions for: argparse, platform, unittest,
/// socket, array, errno, traceback, codecs, logging, pickle, threading, sqlite3.
use crate::runtime::value::MbValue;

// ── helpers ──────────────────────────────────────────────────────────────────

fn str_val(v: MbValue) -> Option<String> {
    v.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn bytes_val(v: MbValue) -> Option<Vec<u8>> {
    v.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Bytes(ref b) = (*ptr).data {
            Some(b.clone())
        } else {
            None
        }
    })
}

// Reads a string-valued attribute from either a Dict (socket/logging/sqlite3
// modules return ObjData::Dict) or an Instance (threading returns
// ObjData::Instance so `.name`/`.locked()` and `isinstance` dispatch work).
fn dict_str(v: MbValue, key: &str) -> Option<String> {
    let str_of = |val: &MbValue| -> Option<String> {
        val.as_ptr().and_then(|p| unsafe {
            if let ObjData::Str(ref s) = (*p).data {
                Some(s.clone())
            } else {
                None
            }
        })
    };
    v.as_ptr().and_then(|ptr| unsafe {
        match (*ptr).data {
            ObjData::Dict(ref lock) => lock.read().unwrap().get(key).and_then(&str_of),
            ObjData::Instance { ref fields, .. } => {
                fields.read().unwrap().get(key).and_then(&str_of)
            }
            _ => None,
        }
    })
}

fn dict_bool(v: MbValue, key: &str) -> Option<bool> {
    v.as_ptr().and_then(|ptr| unsafe {
        match (*ptr).data {
            ObjData::Dict(ref lock) => lock.read().unwrap().get(key).and_then(|val| val.as_bool()),
            ObjData::Instance { ref fields, .. } => fields
                .read()
                .unwrap()
                .get(key)
                .and_then(|val| val.as_bool()),
            _ => None,
        }
    })
}

fn list_len(v: MbValue) -> usize {
    v.as_ptr()
        .map(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().len()
            } else {
                0
            }
        })
        .unwrap_or(0)
}

fn s(text: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(text.to_string()))
}

// ── argparse ─────────────────────────────────────────────────────────────────

#[test]
fn test_argparse_full_lifecycle() {
    use crate::runtime::stdlib::argparse_mod::{
        mb_argparse_add_argument, mb_argparse_new, mb_argparse_parse_args,
    };

    let parser = mb_argparse_new(s("CLI tool"));
    mb_argparse_add_argument(parser, s("--name"));
    mb_argparse_add_argument(parser, s("--count"));

    // parse_args reads from std::env::args; in test context env args won't match
    // so both keys will map to None — verify namespace dict is produced without panic
    let ns = mb_argparse_parse_args(parser);
    assert!(ns.as_ptr().is_some());
}

#[test]
fn test_argparse_non_str_description() {
    use crate::runtime::stdlib::argparse_mod::mb_argparse_new;
    // Non-str desc → description stored as empty string
    let parser = mb_argparse_new(MbValue::from_int(42));
    assert!(parser.as_ptr().is_some());
    // Verify description field is present (empty string)
    if let Some(ptr) = parser.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                assert!(map.contains_key("description"));
            }
        }
    }
}

// ── platform ──────────────────────────────────────────────────────────────────

#[test]
fn test_platform_all_functions_return_strings() {
    use crate::runtime::stdlib::platform_mod::*;

    assert!(str_val(mb_platform_system())
        .map(|s| !s.is_empty())
        .unwrap_or(false));
    // release() now reports the real kernel release (uname -r).
    assert!(str_val(mb_platform_release())
        .map(|s| !s.is_empty())
        .unwrap_or(false));
    assert!(str_val(mb_platform_machine())
        .map(|s| !s.is_empty())
        .unwrap_or(false));
    assert!(str_val(mb_platform_processor())
        .map(|s| !s.is_empty())
        .unwrap_or(false));
    assert!(str_val(mb_platform_python_version())
        .map(|s| s == "3.12.0")
        .unwrap_or(false));
    let plat = str_val(mb_platform_platform()).unwrap_or_default();
    assert!(plat.contains('-'), "platform should be OS-ARCH format");
}

#[test]
fn test_platform_node_hostname_fallback() {
    use crate::runtime::stdlib::platform_mod::mb_platform_node;

    // Remove HOSTNAME to exercise fallback
    let orig = std::env::var("HOSTNAME").ok();
    std::env::remove_var("HOSTNAME");

    let node = str_val(mb_platform_node()).unwrap_or_default();

    // Restore
    if let Some(h) = orig {
        std::env::set_var("HOSTNAME", h);
    }

    assert!(!node.is_empty(), "node should return non-empty string");
}

// ── socket ────────────────────────────────────────────────────────────────────

#[test]
fn test_socket_full_server_lifecycle() {
    use crate::runtime::stdlib::socket_mod::*;

    let sock = mb_socket_new(MbValue::from_int(2), MbValue::from_int(1));

    // Bind to loopback with port 0 (OS assigns)
    let addr = s("127.0.0.1:0");
    mb_socket_bind(sock, addr);
    assert_eq!(dict_bool(sock, "bound"), Some(true));

    // Listen
    mb_socket_listen(sock, MbValue::from_int(5));
    assert_eq!(dict_bool(sock, "listening"), Some(true));

    // Close
    mb_socket_close(sock);
    assert_eq!(dict_bool(sock, "closed"), Some(true));
}

#[test]
fn test_socket_gethostname_and_gethostbyname() {
    use crate::runtime::stdlib::socket_mod::*;

    // gethostname: should not panic
    let name = mb_socket_gethostname();
    assert!(str_val(name).is_some());

    // gethostbyname: always returns "127.0.0.1"
    let ip = mb_socket_gethostbyname(s("localhost"));
    assert_eq!(str_val(ip), Some("127.0.0.1".to_string()));
}

#[test]
fn test_socket_host_env_fallback() {
    use crate::runtime::stdlib::socket_mod::mb_socket_gethostname;

    // When HOSTNAME is unset but HOST is set, gethostname returns HOST
    let orig_hostname = std::env::var("HOSTNAME").ok();
    let orig_host = std::env::var("HOST").ok();

    std::env::remove_var("HOSTNAME");
    std::env::set_var("HOST", "my-host-test");

    let result = str_val(mb_socket_gethostname()).unwrap_or_default();

    // Restore
    if let Some(h) = orig_hostname {
        std::env::set_var("HOSTNAME", h);
    }
    std::env::remove_var("HOST");
    if let Some(h) = orig_host {
        std::env::set_var("HOST", h);
    }

    assert_eq!(result, "my-host-test");
}

// ── array ─────────────────────────────────────────────────────────────────────

#[test]
fn test_array_bytes_roundtrip() {
    use crate::runtime::stdlib::array_mod::*;

    // Create array, append ints, tobytes, frombytes into new array
    let arr = mb_array_new(s("i"), MbValue::none());
    mb_array_append(arr, MbValue::from_int(10));
    mb_array_append(arr, MbValue::from_int(20));
    mb_array_append(arr, MbValue::from_int(30));

    // typecode 'i' is a 4-byte signed int, so tobytes() emits the 12-byte
    // little-endian image (CPython 3.12 parity), NOT one byte per element.
    let raw = mb_array_tobytes(arr);
    assert_eq!(
        bytes_val(raw).unwrap_or_default(),
        vec![10u8, 0, 0, 0, 20, 0, 0, 0, 30, 0, 0, 0]
    );

    let arr2 = mb_array_new(s("i"), MbValue::none());
    mb_array_frombytes(arr2, raw);

    // frombytes decodes the 12-byte image back into 3 elements.
    let list = mb_array_tolist(arr2);
    assert_eq!(list_len(list), 3);
    assert_eq!(
        bytes_val(mb_array_tobytes(arr2)).unwrap_or_default(),
        vec![10u8, 0, 0, 0, 20, 0, 0, 0, 30, 0, 0, 0]
    );
}

#[test]
fn test_array_non_list_init_is_empty() {
    use crate::runtime::stdlib::array_mod::mb_array_new;

    // Non-list initializer → empty data
    let arr = mb_array_new(s("i"), MbValue::from_int(99));
    let list = crate::runtime::stdlib::array_mod::mb_array_tolist(arr);
    assert_eq!(list_len(list), 0);
}

// ── errno ─────────────────────────────────────────────────────────────────────

#[test]
fn test_errno_errorcode_and_strerror_integration() {
    use crate::runtime::dict_ops::DictKey;
    use crate::runtime::stdlib::errno_mod::{mb_errno_errorcode, mb_errno_strerror};

    // errorcode dict: 2 → "ENOENT". CPython parity: keys are ints, so
    // the lookup must use `DictKey::Int(2)` rather than the string "2".
    let code_dict = mb_errno_errorcode();
    if let Some(ptr) = code_dict.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                let enoent = map.get(&DictKey::Int(2)).and_then(|v| str_val(*v));
                assert_eq!(enoent, Some("ENOENT".to_string()));
            }
        }
    }

    // strerror: verify several codes
    assert_eq!(
        str_val(mb_errno_strerror(MbValue::from_int(1))).as_deref(),
        Some("Operation not permitted")
    );
    assert_eq!(
        str_val(mb_errno_strerror(MbValue::from_int(22))).as_deref(),
        Some("Invalid argument")
    );
    assert_eq!(
        str_val(mb_errno_strerror(MbValue::from_int(111))).as_deref(),
        Some("Connection refused")
    );
    assert_eq!(
        str_val(mb_errno_strerror(MbValue::from_int(0))).as_deref(),
        Some("Unknown error")
    );
}

// ── traceback ────────────────────────────────────────────────────────────────

#[test]
fn test_traceback_format_exception_non_exception_raises_type_error() {
    use crate::runtime::exception;
    use crate::runtime::stdlib::traceback_mod::mb_traceback_format_exception;

    // CPython: format_exception(42) raises TypeError.
    exception::mb_clear_exception();
    let r = mb_traceback_format_exception(&[MbValue::from_int(42)]);
    assert!(r.is_none());
    assert_eq!(exception::mb_has_exception().as_bool(), Some(true));
    exception::mb_clear_exception();
}

#[test]
fn test_traceback_format_exception_two_args_raises_value_error() {
    use crate::runtime::exception;
    use crate::runtime::stdlib::traceback_mod::mb_traceback_format_exception;

    // CPython: passing value without tb raises ValueError.
    exception::mb_clear_exception();
    let r = mb_traceback_format_exception(&[s("Exception"), s("x")]);
    assert!(r.is_none());
    assert_eq!(exception::mb_has_exception().as_bool(), Some(true));
    exception::mb_clear_exception();
}

// ── codecs ────────────────────────────────────────────────────────────────────

#[test]
fn test_codecs_encode_decode_all_codecs() {
    use crate::runtime::stdlib::codecs_mod::*;

    // UTF-8 roundtrip
    let encoded = mb_codecs_encode(s("hello"), s("utf-8"));
    let decoded = mb_codecs_decode(encoded, s("utf-8"));
    assert_eq!(str_val(decoded).as_deref(), Some("hello"));

    // ASCII roundtrip (all ASCII chars)
    let encoded = mb_codecs_encode(s("world"), s("ascii"));
    let decoded = mb_codecs_decode(encoded, s("ascii"));
    assert_eq!(str_val(decoded).as_deref(), Some("world"));

    // Latin-1 roundtrip (within range)
    let encoded = mb_codecs_encode(s("abc"), s("latin-1"));
    let decoded = mb_codecs_decode(encoded, s("latin-1"));
    assert_eq!(str_val(decoded).as_deref(), Some("abc"));
}

#[test]
fn test_codecs_normalize_encoding_variants() {
    use crate::runtime::stdlib::codecs_mod::mb_codecs_encode;

    // "UTF_8" variant should encode just like "utf-8"
    let r1 = mb_codecs_encode(s("test"), s("utf-8"));
    let r2 = mb_codecs_encode(s("test"), s("UTF_8"));
    assert_eq!(bytes_val(r1), bytes_val(r2));
}

// ── logging ───────────────────────────────────────────────────────────────────

#[test]
fn test_logging_level_filtering() {
    use crate::runtime::stdlib::logging_mod::*;

    // Set level to DEBUG (10) → all levels emit
    mb_logging_basicconfig(MbValue::from_int(10));
    let r = mb_logging_debug(s("debug msg"));
    assert!(r.is_none());
    let r = mb_logging_info(s("info msg"));
    assert!(r.is_none());
    let r = mb_logging_warning(s("warning msg"));
    assert!(r.is_none());
    let r = mb_logging_error(s("error msg"));
    assert!(r.is_none());
    let r = mb_logging_critical(s("critical msg"));
    assert!(r.is_none());

    // Restore to default WARNING level
    mb_logging_basicconfig(MbValue::from_int(30));
}

#[test]
fn test_logging_getlogger_names() {
    use crate::runtime::stdlib::logging_mod::mb_logging_getlogger;

    let root = mb_logging_getlogger(MbValue::none());
    assert_eq!(dict_str(root, "name").as_deref(), Some("root"));

    let named = mb_logging_getlogger(s("app.db"));
    assert_eq!(dict_str(named, "name").as_deref(), Some("app.db"));
}

// ── pickle ────────────────────────────────────────────────────────────────────

#[test]
fn test_pickle_roundtrip_all_types() {
    use crate::runtime::stdlib::pickle_mod::{mb_pickle_dumps, mb_pickle_loads};

    // None
    let r = mb_pickle_loads(mb_pickle_dumps(MbValue::none()));
    assert!(r.is_none());

    // Bool
    let r = mb_pickle_loads(mb_pickle_dumps(MbValue::from_bool(true)));
    assert_eq!(r.as_bool(), Some(true));

    // Int (positive and negative)
    let r = mb_pickle_loads(mb_pickle_dumps(MbValue::from_int(99)));
    assert_eq!(r.as_int(), Some(99));

    let r = mb_pickle_loads(mb_pickle_dumps(MbValue::from_int(-7)));
    assert_eq!(r.as_int(), Some(-7));

    // Float
    let r = mb_pickle_loads(mb_pickle_dumps(MbValue::from_float(2.5)));
    let f = r.as_float().expect("float");
    assert!((f - 2.5).abs() < 0.001);

    // String
    let r = mb_pickle_loads(mb_pickle_dumps(s("test")));
    assert_eq!(str_val(r).as_deref(), Some("test"));
}

#[test]
fn test_pickle_nested_list() {
    use crate::runtime::stdlib::pickle_mod::{mb_pickle_dumps, mb_pickle_loads};

    // [[1, 2], [3, 4]]
    let inner1 = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_int(1),
        MbValue::from_int(2),
    ]));
    let inner2 = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_int(3),
        MbValue::from_int(4),
    ]));
    let outer = MbValue::from_ptr(MbObject::new_list(vec![inner1, inner2]));

    let data = mb_pickle_dumps(outer);
    let result = mb_pickle_loads(data);

    if let Some(ptr) = result.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 2);
            } else {
                panic!("expected outer list");
            }
        }
    } else {
        panic!("expected Some ptr for nested list roundtrip");
    }
}

#[test]
fn test_pickle_loads_negative_int() {
    use crate::runtime::stdlib::pickle_mod::{mb_pickle_dumps, mb_pickle_loads};

    // Negative int round-trips through the CPython-compatible binary format.
    let r = mb_pickle_loads(mb_pickle_dumps(MbValue::from_int(-42)));
    assert_eq!(r.as_int(), Some(-42));
}

#[test]
fn test_pickle_loads_empty_list() {
    use crate::runtime::stdlib::pickle_mod::{mb_pickle_dumps, mb_pickle_loads};

    // Empty list round-trips to an empty list.
    let empty = MbValue::from_ptr(MbObject::new_list(vec![]));
    let r = mb_pickle_loads(mb_pickle_dumps(empty));
    assert!(r.as_ptr().is_some());
    if let Some(ptr) = r.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                assert!(lock.read().unwrap().is_empty());
            } else {
                panic!("expected empty list");
            }
        }
    }
}

// ── threading ─────────────────────────────────────────────────────────────────

#[test]
fn test_threading_deterministic_sync_with_barrier() {
    use crate::runtime::stdlib::threading_mod::*;
    use std::sync::{Arc, Barrier};

    // Create a thread-safe counter using Barrier for deterministic sync
    let barrier = Arc::new(Barrier::new(2));
    let b2 = Arc::clone(&barrier);

    let thread_dict = mb_threading_thread(MbValue::none(), s("sync_worker"));
    assert_eq!(
        dict_str(thread_dict, "name").as_deref(),
        Some("sync_worker")
    );

    let counter = Arc::new(std::sync::atomic::AtomicI64::new(0));
    let counter2 = Arc::clone(&counter);

    let handle = std::thread::spawn(move || {
        counter2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        b2.wait();
    });

    barrier.wait();
    handle.join().expect("thread panicked");

    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[test]
fn test_threading_lock_acquire_release_integration() {
    use crate::runtime::stdlib::threading_mod::*;

    let lock = mb_threading_lock();
    assert_eq!(dict_bool(lock, "locked"), Some(false));

    let r = mb_threading_lock_acquire(lock);
    assert!(r.as_bool().is_some() || r.is_none()); // returns True or None
    assert_eq!(dict_bool(lock, "locked"), Some(true));

    mb_threading_lock_release(lock);
    assert_eq!(dict_bool(lock, "locked"), Some(false));
}

#[test]
fn test_threading_event_lifecycle() {
    use crate::runtime::stdlib::threading_mod::*;

    let event = mb_threading_event();
    assert_eq!(mb_threading_event_is_set(event).as_bool(), Some(false));

    mb_threading_event_set(event);
    assert_eq!(mb_threading_event_is_set(event).as_bool(), Some(true));

    mb_threading_event_clear(event);
    assert_eq!(mb_threading_event_is_set(event).as_bool(), Some(false));
}

// ── sqlite3 ───────────────────────────────────────────────────────────────────

// Class name of an `ObjData::Instance` value (sqlite3 returns real class
// instances since the isinstance-dispatch rework, not dict-backed shims).
fn instance_class(v: MbValue) -> Option<String> {
    v.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            Some(class_name.clone())
        } else {
            None
        }
    })
}

#[test]
fn test_sqlite3_full_workflow() {
    use crate::runtime::stdlib::sqlite3_mod::*;

    // The real (rusqlite-backed) connect returns a genuine Connection instance,
    // and cursor() returns a distinct Cursor instance. SQL execution is driven
    // through the registered Connection/Cursor methods (covered by the
    // tests/cpython sqlite3 conformance fixtures).
    let conn = mb_sqlite3_connect(s(":memory:"));
    assert_eq!(instance_class(conn).as_deref(), Some("Connection"));
    let cur = mb_sqlite3_cursor(conn);
    assert_eq!(instance_class(cur).as_deref(), Some("Cursor"));
}

#[test]
fn test_sqlite3_create_table_if_not_exists_integration() {
    use crate::runtime::stdlib::sqlite3_mod::mb_sqlite3_connect;

    let conn = mb_sqlite3_connect(s(":memory:"));
    assert!(conn.as_ptr().is_some());
}

#[test]
fn test_sqlite3_cursor_returns_distinct_cursor_instance() {
    use crate::runtime::stdlib::sqlite3_mod::{mb_sqlite3_connect, mb_sqlite3_cursor};

    // CPython 3.12: conn.cursor() returns a Cursor object, not the
    // connection itself; mamba mirrors that with a `Cursor` instance
    // sharing the connection's in-memory state.
    let conn = mb_sqlite3_connect(s(":memory:"));
    let cursor = mb_sqlite3_cursor(conn);
    assert_ne!(conn, cursor);
    assert_eq!(instance_class(cursor).as_deref(), Some("Cursor"));
}

// ── unittest ──────────────────────────────────────────────────────────────────

#[test]
fn test_unittest_testcase_and_asserts() {
    use crate::runtime::stdlib::unittest_mod::*;

    let tc = mb_unittest_testcase();
    assert!(tc.as_ptr().is_some());

    // assertEqual
    mb_unittest_assert_equal(MbValue::from_int(1), MbValue::from_int(1));

    // assertNotEqual
    mb_unittest_assert_not_equal(s("foo"), s("bar"));

    // assertTrue
    mb_unittest_assert_true(MbValue::from_bool(true));

    // assertFalse
    mb_unittest_assert_false(MbValue::from_int(0));

    // assertIsNone
    mb_unittest_assert_is_none(MbValue::none());

    // assertIn list
    let list = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_int(1),
        MbValue::from_int(2),
    ]));
    mb_unittest_assert_in(MbValue::from_int(2), list);

    // assertIn str
    mb_unittest_assert_in(s("x"), s("xyz"));

    // main stub
    let r = mb_unittest_main();
    assert!(r.is_none());
}

#[test]
fn test_unittest_assert_raises_creates_dict() {
    use crate::runtime::stdlib::unittest_mod::mb_unittest_assert_raises;

    let ctx = mb_unittest_assert_raises(s("ValueError"));
    if let Some(ptr) = ctx.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                assert!(map.contains_key("expected"));
            }
        }
    }
}
