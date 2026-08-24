use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/selectors/get_key_missing_raises_keyerror.py`.
#[test]
fn test_gen_errors_std_libs_selectors_get_key_missing_raises_keyerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "errors"
# case = "get_key_missing_raises_keyerror"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: get_key() on a never-registered file object raises KeyError"""
import selectors

with selectors.DefaultSelector() as _sel:
    _raised = False
    try:
        _sel.get_key(99999)
    except KeyError:
        _raised = True
    assert _raised, "get_key of a never-registered file object must raise KeyError"
print("get_key_missing_raises_keyerror OK")
"###);
    assert_output(&out, r###"get_key_missing_raises_keyerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/selectors/modify_missing_raises_keyerror.py`.
#[test]
fn test_gen_errors_std_libs_selectors_modify_missing_raises_keyerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "errors"
# case = "modify_missing_raises_keyerror"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: modify() on a never-registered socket raises KeyError"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _raised = False
    try:
        _sel.modify(_s, selectors.EVENT_READ)
    except KeyError:
        _raised = True
    assert _raised, "modify of a never-registered socket must raise KeyError"
_s.close()
print("modify_missing_raises_keyerror OK")
"###);
    assert_output(&out, r###"modify_missing_raises_keyerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/selectors/register_bad_events_raises.py`.
#[test]
fn test_gen_errors_std_libs_selectors_register_bad_events_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "errors"
# case = "register_bad_events_raises"
# subject = "selectors.DefaultSelector"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: register_bad_events_raises (errors)."""
import selectors
import socket

_raised = False
try:
    selectors.DefaultSelector().register(socket.socket(), 999, None)
except ValueError:
    _raised = True
assert _raised, "register_bad_events_raises: expected ValueError"
print("register_bad_events_raises OK")
"###);
    assert_output(&out, r###"register_bad_events_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/selectors/register_duplicate_raises_keyerror.py`.
#[test]
fn test_gen_errors_std_libs_selectors_register_duplicate_raises_keyerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "errors"
# case = "register_duplicate_raises_keyerror"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: registering the same socket twice on one DefaultSelector raises KeyError on the second register"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _sel.register(_s, selectors.EVENT_READ)
    _raised = False
    try:
        _sel.register(_s, selectors.EVENT_WRITE)
    except KeyError:
        _raised = True
    assert _raised, "second register of same socket must raise KeyError"
_s.close()
print("register_duplicate_raises_keyerror OK")
"###);
    assert_output(&out, r###"register_duplicate_raises_keyerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/selectors/register_negative_fd_raises.py`.
#[test]
fn test_gen_errors_std_libs_selectors_register_negative_fd_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "errors"
# case = "register_negative_fd_raises"
# subject = "selectors.DefaultSelector"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: register_negative_fd_raises (errors)."""
import selectors

_raised = False
try:
    selectors.DefaultSelector().register(-10, selectors.EVENT_READ)
except ValueError:
    _raised = True
assert _raised, "register_negative_fd_raises: expected ValueError"
print("register_negative_fd_raises OK")
"###);
    assert_output(&out, r###"register_negative_fd_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/selectors/select_zero_timeout_returns_list.py`.
#[test]
fn test_gen_errors_std_libs_selectors_select_zero_timeout_returns_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "errors"
# case = "select_zero_timeout_returns_list"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""selectors.DefaultSelector: select(timeout=0) on a selector with no ready fds returns an empty list without blocking or raising"""
import selectors

with selectors.DefaultSelector() as _sel:
    _result = _sel.select(timeout=0)
    assert isinstance(_result, list), f"select() must return a list, got {type(_result)!r}"
    assert _result == [], f"empty selector select(timeout=0) must be [], got {_result!r}"
print("select_zero_timeout_returns_list OK")
"###);
    assert_output(&out, r###"select_zero_timeout_returns_list OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/selectors/unregister_missing_raises_keyerror.py`.
#[test]
fn test_gen_errors_std_libs_selectors_unregister_missing_raises_keyerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "errors"
# case = "unregister_missing_raises_keyerror"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: unregistering a never-registered socket raises KeyError"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _raised = False
    try:
        _sel.unregister(_s)
    except KeyError:
        _raised = True
    assert _raised, "unregister of a never-registered socket must raise KeyError"
_s.close()
print("unregister_missing_raises_keyerror OK")
"###);
    assert_output(&out, r###"unregister_missing_raises_keyerror OK
"###);
}
