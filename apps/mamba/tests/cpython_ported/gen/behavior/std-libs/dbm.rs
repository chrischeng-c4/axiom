use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/dbm/data_persists_across_reopen.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_data_persists_across_reopen() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "data_persists_across_reopen"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.open: data written under mode 'c' survives close and is readable after reopen in mode 'r'"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["persist_key"] = "persist_value"
    # Reopen and verify.
    with dbm.open(_path, "r") as _db:
        assert "persist_key" in _db, "key persisted"
        assert _db["persist_key"] == b"persist_value", f"value persisted: {_db['persist_key']!r}"
print("data_persists_across_reopen OK")
"###);
    assert_output(&out, r###"data_persists_across_reopen OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dbm/del_removes_key.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_del_removes_key() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "del_removes_key"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.open: del db[key] removes only that key; the remaining keys stay present"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["a"] = "1"
        _db["b"] = "2"
        del _db["a"]
        assert "a" not in _db, "a deleted"
        assert "b" in _db, "b remains"
print("del_removes_key OK")
"###);
    assert_output(&out, r###"del_removes_key OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dbm/empty_bytes_value_distinct_from_absent.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_empty_bytes_value_distinct_from_absent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "empty_bytes_value_distinct_from_absent"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: an empty-bytes value is stored and present, and stays distinguishable from an absent key (get -> None)"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db[b"empty"] = b""
        assert b"empty" in _db, "empty-valued key is present"
        assert _db[b"empty"] == b"", f"empty value = {_db[b'empty']!r}"
        assert _db.get(b"empty") == b"", "get returns empty bytes"
        # An absent key is still distinguishable from an empty value.
        assert _db.get(b"never") is None, "absent key still None"
print("empty_bytes_value_distinct_from_absent OK")
"###);
    assert_output(&out, r###"empty_bytes_value_distinct_from_absent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dbm/get_returns_none_for_missing.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_get_returns_none_for_missing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "get_returns_none_for_missing"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: get returns the stored value for a present key, None for an absent key, and the supplied default otherwise"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["present"] = "yes"
        assert _db.get("present") == b"yes", "get present"
        assert _db.get("absent") is None, "get absent is None"
        assert _db.get("absent", b"fallback") == b"fallback", "get absent default"
print("get_returns_none_for_missing OK")
"###);
    assert_output(&out, r###"get_returns_none_for_missing OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dbm/items_yields_all_pairs.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_items_yields_all_pairs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "items_yields_all_pairs"
# subject = "dbm.dumb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb: items() yields every stored (key, value) pair as bytes (dumb backend)"""
import dbm.dumb
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.dumb.open(_path, "c") as _db:
        _db[b"a"] = b"1"
        _db[b"b"] = b"2"
    with dbm.dumb.open(_path, "r") as _db:
        assert sorted(_db.items()) == [(b"a", b"1"), (b"b", b"2")], \
            f"items round-trip = {sorted(_db.items())!r}"
print("items_yields_all_pairs OK")
"###);
    assert_output(&out, r###"items_yields_all_pairs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dbm/keys_iteration_matches_inserts.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_keys_iteration_matches_inserts() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "keys_iteration_matches_inserts"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: keys() after reopen reports exactly the inserted keys (as bytes), regardless of order"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    _data = {"alpha": "A", "beta": "B", "gamma": "C"}
    with dbm.open(_path, "c") as _db:
        for _k, _v in _data.items():
            _db[_k] = _v
    with dbm.open(_path, "r") as _db:
        _keys = set(_db.keys())
        assert len(_keys) == 3, f"three keys = {len(_keys)!r}"
        for _k in _data:
            assert _k.encode() in _keys, f"{_k!r} in keys"
print("keys_iteration_matches_inserts OK")
"###);
    assert_output(&out, r###"keys_iteration_matches_inserts OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dbm/len_reports_live_key_count.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_len_reports_live_key_count() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "len_reports_live_key_count"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.open: len(db) reflects the live key count across inserts and a delete"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        assert len(_db) == 0, f"empty len = {len(_db)!r}"
        _db["a"] = "1"
        _db["b"] = "2"
        assert len(_db) == 2, f"len after 2 inserts = {len(_db)!r}"
        del _db["a"]
        assert len(_db) == 1, f"len after del = {len(_db)!r}"
print("len_reports_live_key_count OK")
"###);
    assert_output(&out, r###"len_reports_live_key_count OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dbm/n_flag_truncates_existing.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_n_flag_truncates_existing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "n_flag_truncates_existing"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.open: opening an existing db with the 'n' flag truncates it to empty before new writes"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["old"] = "data"
    # Opening with 'n' truncates the existing db.
    with dbm.open(_path, "n") as _db:
        assert "old" not in _db, "old data gone after 'n' open"
        _db["new"] = "fresh"
    with dbm.open(_path, "r") as _db:
        assert "new" in _db, "new data present"
        assert "old" not in _db, "old data still gone"
print("n_flag_truncates_existing OK")
"###);
    assert_output(&out, r###"n_flag_truncates_existing OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dbm/overwrite_replaces_value.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_overwrite_replaces_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "overwrite_replaces_value"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: overwriting a key replaces its value and keeps the db at one key (CPython bug #482460)"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db[b"1"] = b"hello"
        _db[b"1"] = b"hello2"
    with dbm.open(_path, "r") as _db:
        assert _db[b"1"] == b"hello2", f"overwrite wins = {_db[b'1']!r}"
        assert len(_db) == 1, f"overwrite keeps one key = {len(_db)!r}"
print("overwrite_replaces_value OK")
"###);
    assert_output(&out, r###"overwrite_replaces_value OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dbm/setdefault_like_dict.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_setdefault_like_dict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "setdefault_like_dict"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: setdefault stores and returns the default for a new key, and keeps the existing value on a second call"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _r = _db.setdefault(b"xxx", b"foo")
        assert _r == b"foo", f"setdefault new returns default = {_r!r}"
        assert _db[b"xxx"] == b"foo", "setdefault stored value"
        # A second call leaves the existing value untouched.
        _r2 = _db.setdefault(b"xxx", b"other")
        assert _r2 == b"foo", f"setdefault keeps existing = {_r2!r}"
print("setdefault_like_dict OK")
"###);
    assert_output(&out, r###"setdefault_like_dict OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dbm/str_and_bytes_keys_same_entry.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_str_and_bytes_keys_same_entry() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "str_and_bytes_keys_same_entry"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.open: a str-written key is reachable via its bytes form, and keys() reports the bytes spelling"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["1"] = "a"            # str key, str value
    with dbm.open(_path, "r") as _db:
        # The same entry is reachable via the bytes form of the key.
        assert _db[b"1"] == b"a", f"str-write/bytes-read = {_db[b'1']!r}"
        assert b"1" in list(_db.keys()), "keys() reports bytes form"
print("str_and_bytes_keys_same_entry OK")
"###);
    assert_output(&out, r###"str_and_bytes_keys_same_entry OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dbm/str_values_stored_as_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_str_values_stored_as_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "str_values_stored_as_bytes"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: str inputs are stored and read back as bytes; bytes keys/values round-trip unchanged"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["key"] = "value"
        _v = _db["key"]
        assert isinstance(_v, bytes), f"value is bytes: {type(_v)!r}"
        assert _v == b"value", f"value = {_v!r}"
        _db[b"bkey"] = b"bval"
        assert _db[b"bkey"] == b"bval", "bytes key/val"
print("str_values_stored_as_bytes OK")
"###);
    assert_output(&out, r###"str_values_stored_as_bytes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dbm/utf8_keys_values_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_utf8_keys_values_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "utf8_keys_values_roundtrip"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.open: a non-ASCII (UTF-8) str key round-trips and is readable via its UTF-8-encoded bytes form"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    _ukey = "ü"  # u-umlaut
    with dbm.open(_path, "c") as _db:
        _db[_ukey] = b"!"
    with dbm.open(_path, "r") as _db:
        assert _ukey in _db, "non-ASCII str key present"
        assert _db[_ukey.encode("utf-8")] == b"!", "non-ASCII bytes key reads"
print("utf8_keys_values_roundtrip OK")
"###);
    assert_output(&out, r###"utf8_keys_values_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dbm/whichdb_detects_dumb_backend.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_whichdb_detects_dumb_backend() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "whichdb_detects_dumb_backend"
# subject = "dbm.whichdb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.whichdb: whichdb identifies a dumb-format db as 'dbm.dumb' for both str and bytes path spellings, even when empty"""
import dbm
import dbm.dumb
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _dbpath = os.path.join(_d, "store")
    # An empty dumb db is still detected as 'dbm.dumb'.
    with dbm.dumb.open(_dbpath, "c") as _f:
        pass
    assert dbm.whichdb(_dbpath) == "dbm.dumb", f"empty dumb db = {dbm.whichdb(_dbpath)!r}"

    with dbm.dumb.open(_dbpath, "w") as _f:
        _f[b"key"] = b"value"
    # The answer is independent of how the path is spelled (str or bytes).
    for _path in (_dbpath, os.fsencode(_dbpath)):
        assert dbm.whichdb(_path) == "dbm.dumb", \
            f"populated dumb db ({type(_path).__name__}) = {dbm.whichdb(_path)!r}"
print("whichdb_detects_dumb_backend OK")
"###);
    assert_output(&out, r###"whichdb_detects_dumb_backend OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dbm/whichdb_empty_db_file_is_none.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_whichdb_empty_db_file_is_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "whichdb_empty_db_file_is_none"
# subject = "dbm.whichdb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.whichdb: whichdb returns None for a non-existent path and for a bare empty .db file (issue 17198)"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    # A non-existent path is recognized by nobody.
    _missing = os.path.join(_d, "missing")
    assert dbm.whichdb(_missing) is None, "whichdb(missing) is None"

    # A bare, empty `.db` file is not a real database -> None (issue 17198:
    # whichdb must not misreport an empty .db as the ndbm backend).
    _bare = os.path.join(_d, "bare")
    with open(_bare + ".db", "wb"):
        pass
    assert dbm.whichdb(_bare) is None, f"empty .db = {dbm.whichdb(_bare)!r}"
    assert dbm.whichdb(os.fsencode(_bare)) is None, "empty .db (bytes path)"
print("whichdb_empty_db_file_is_none OK")
"###);
    assert_output(&out, r###"whichdb_empty_db_file_is_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/dbm/whichdb_reports_module_name.py`.
#[test]
fn test_gen_behavior_std_libs_dbm_whichdb_reports_module_name() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "whichdb_reports_module_name"
# subject = "dbm.whichdb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.whichdb: whichdb returns a non-empty backend module name (str) for an existing db"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["x"] = "y"
    _which = dbm.whichdb(_path)
    assert _which is not None, f"whichdb returns module: {_which!r}"
    assert isinstance(_which, str), f"whichdb type = {type(_which)!r}"
    assert _which != "", "whichdb name non-empty"
print("whichdb_reports_module_name OK")
"###);
    assert_output(&out, r###"whichdb_reports_module_name OK
"###);
}
