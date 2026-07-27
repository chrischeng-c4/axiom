use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/dbm/dumb_contains_on_closed_raises.py`.
#[test]
fn test_gen_errors_std_libs_dbm_dumb_contains_on_closed_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "dumb_contains_on_closed_raises"
# subject = "dbm.dumb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb: membership test on a closed dumb handle raises dbm.dumb.error with the fixed 'already been closed' message"""
import dbm.dumb
import os
import tempfile

_closed_msg = "DBM object has already been closed"
with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    _f = dbm.dumb.open(_path, "c")
    _f.close()
    _raised = False
    try:
        _ = b"test" in _f
    except dbm.dumb.error as e:
        _raised = True
        assert str(e) == _closed_msg, f"contains closed msg = {str(e)!r}"
    assert _raised, "contains on closed db must raise"
print("dumb_contains_on_closed_raises OK")
"###);
    assert_output(&out, r###"dumb_contains_on_closed_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/dbm/dumb_double_close_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_dbm_dumb_double_close_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "dumb_double_close_no_raise"
# subject = "dbm.dumb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb: closing an already-closed dumb handle a second time is a no-op and does not raise"""
import dbm.dumb
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    _f = dbm.dumb.open(_path, "c")
    _f.close()
    # Double close is a no-op (must not raise).
    _f.close()
print("dumb_double_close_no_raise OK")
"###);
    assert_output(&out, r###"dumb_double_close_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/dbm/dumb_getitem_on_closed_raises.py`.
#[test]
fn test_gen_errors_std_libs_dbm_dumb_getitem_on_closed_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "dumb_getitem_on_closed_raises"
# subject = "dbm.dumb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb: getitem on a closed dumb handle raises dbm.dumb.error with the fixed 'already been closed' message"""
import dbm.dumb
import os
import tempfile

_closed_msg = "DBM object has already been closed"
with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    _f = dbm.dumb.open(_path, "c")
    _f.close()
    _raised = False
    try:
        _ = _f[b"test"]
    except dbm.dumb.error as e:
        _raised = True
        assert str(e) == _closed_msg, f"getitem closed msg = {str(e)!r}"
    assert _raised, "getitem on closed db must raise"
print("dumb_getitem_on_closed_raises OK")
"###);
    assert_output(&out, r###"dumb_getitem_on_closed_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/dbm/dumb_invalid_flag_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_dbm_dumb_invalid_flag_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "dumb_invalid_flag_raises_valueerror"
# subject = "dbm.dumb.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb.open: dumb_invalid_flag_raises_valueerror (errors)."""
import dbm.dumb

_raised = False
try:
    dbm.dumb.open("/tmp/__mamba_dbm_unused__/db", "q")
except ValueError:
    _raised = True
assert _raised, "dumb_invalid_flag_raises_valueerror: expected ValueError"
print("dumb_invalid_flag_raises_valueerror OK")
"###);
    assert_output(&out, r###"dumb_invalid_flag_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/dbm/dumb_keys_on_closed_raises.py`.
#[test]
fn test_gen_errors_std_libs_dbm_dumb_keys_on_closed_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "dumb_keys_on_closed_raises"
# subject = "dbm.dumb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb: keys() on a closed dumb handle raises dbm.dumb.error with the fixed 'already been closed' message"""
import dbm.dumb
import os
import tempfile

_closed_msg = "DBM object has already been closed"
with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    _f = dbm.dumb.open(_path, "c")
    _f.close()
    _raised = False
    try:
        _f.keys()
    except dbm.dumb.error as e:
        _raised = True
        assert str(e) == _closed_msg, f"keys closed msg = {str(e)!r}"
    assert _raised, "keys on closed db must raise"
print("dumb_keys_on_closed_raises OK")
"###);
    assert_output(&out, r###"dumb_keys_on_closed_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/dbm/dumb_len_on_closed_raises.py`.
#[test]
fn test_gen_errors_std_libs_dbm_dumb_len_on_closed_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "dumb_len_on_closed_raises"
# subject = "dbm.dumb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb: len() on a closed dumb handle raises dbm.dumb.error with the fixed 'already been closed' message"""
import dbm.dumb
import os
import tempfile

_closed_msg = "DBM object has already been closed"
with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    _f = dbm.dumb.open(_path, "c")
    _f.close()
    _raised = False
    try:
        len(_f)
    except dbm.dumb.error as e:
        _raised = True
        assert str(e) == _closed_msg, f"len closed msg = {str(e)!r}"
    assert _raised, "len on closed db must raise"
print("dumb_len_on_closed_raises OK")
"###);
    assert_output(&out, r###"dumb_len_on_closed_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/dbm/dumb_readonly_del_raises.py`.
#[test]
fn test_gen_errors_std_libs_dbm_dumb_readonly_del_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "dumb_readonly_del_raises"
# subject = "dbm.dumb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb: deleting through a read-only dumb handle raises dbm.dumb.error with the fixed 'opened for reading only' message"""
import dbm.dumb
import os
import tempfile

_ro_msg = "The database is opened for reading only"
with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.dumb.open(_path, "c") as _w:
        _w[b"a"] = b"1"
    with dbm.dumb.open(_path, "r") as _r:
        _raised = False
        try:
            del _r[b"a"]
        except dbm.dumb.error as e:
            _raised = True
            assert str(e) == _ro_msg, f"ro del msg = {str(e)!r}"
        assert _raised, "delete on read-only db must raise"
print("dumb_readonly_del_raises OK")
"###);
    assert_output(&out, r###"dumb_readonly_del_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/dbm/dumb_readonly_write_raises.py`.
#[test]
fn test_gen_errors_std_libs_dbm_dumb_readonly_write_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "dumb_readonly_write_raises"
# subject = "dbm.dumb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb: writing through a read-only dumb handle raises dbm.dumb.error with the fixed 'opened for reading only' message"""
import dbm.dumb
import os
import tempfile

_ro_msg = "The database is opened for reading only"
with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.dumb.open(_path, "c") as _w:
        _w[b"a"] = b"1"
    with dbm.dumb.open(_path, "r") as _r:
        _raised = False
        try:
            _r[b"g"] = b"x"
        except dbm.dumb.error as e:
            _raised = True
            assert str(e) == _ro_msg, f"ro write msg = {str(e)!r}"
        assert _raised, "write to read-only db must raise"
print("dumb_readonly_write_raises OK")
"###);
    assert_output(&out, r###"dumb_readonly_write_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/dbm/dumb_setitem_on_closed_raises.py`.
#[test]
fn test_gen_errors_std_libs_dbm_dumb_setitem_on_closed_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "dumb_setitem_on_closed_raises"
# subject = "dbm.dumb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.dumb: setitem on a closed dumb handle raises dbm.dumb.error with the fixed 'already been closed' message"""
import dbm.dumb
import os
import tempfile

_closed_msg = "DBM object has already been closed"
with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    _f = dbm.dumb.open(_path, "c")
    _f.close()
    _raised = False
    try:
        _f[b"test"] = b"v"
    except dbm.dumb.error as e:
        _raised = True
        assert str(e) == _closed_msg, f"setitem closed msg = {str(e)!r}"
    assert _raised, "setitem on closed db must raise"
print("dumb_setitem_on_closed_raises OK")
"###);
    assert_output(&out, r###"dumb_setitem_on_closed_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/dbm/missing_key_raises_keyerror.py`.
#[test]
fn test_gen_errors_std_libs_dbm_missing_key_raises_keyerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "missing_key_raises_keyerror"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: indexing an absent key on an open db raises KeyError"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["exists"] = "yes"
        _raised = False
        try:
            _ = _db["missing"]
        except KeyError:
            _raised = True
        assert _raised, "missing key raises KeyError"
print("missing_key_raises_keyerror OK")
"###);
    assert_output(&out, r###"missing_key_raises_keyerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/dbm/open_missing_read_raises_error.py`.
#[test]
fn test_gen_errors_std_libs_dbm_open_missing_read_raises_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "open_missing_read_raises_error"
# subject = "dbm.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
"""dbm.open: open_missing_read_raises_error (errors)."""
import dbm

_raised = False
try:
    dbm.open("/no/such/dir/no_such_dbm_path_xyz_qwer", "r")
except dbm.error:
    _raised = True
assert _raised, "open_missing_read_raises_error: expected dbm.error"
print("open_missing_read_raises_error OK")
"###);
    assert_output(&out, r###"open_missing_read_raises_error OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/dbm/write_to_readonly_raises.py`.
#[test]
fn test_gen_errors_std_libs_dbm_write_to_readonly_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "errors"
# case = "write_to_readonly_raises"
# subject = "dbm.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.open: writing a key to a db opened in read mode ('r') raises"""
import dbm
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "db")
    with dbm.open(_path, "c") as _db:
        _db["init"] = "data"
    with dbm.open(_path, "r") as _db:
        _raised = False
        try:
            _db["new"] = "value"
        except Exception:
            _raised = True
        assert _raised, "write to read-only db raises"
print("write_to_readonly_raises OK")
"###);
    assert_output(&out, r###"write_to_readonly_raises OK
"###);
}
