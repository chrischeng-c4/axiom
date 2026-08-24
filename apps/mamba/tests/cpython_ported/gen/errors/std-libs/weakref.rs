use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/weakref/hash_proxy_raises.py`.
#[test]
fn test_gen_errors_std_libs_weakref_hash_proxy_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "hash_proxy_raises"
# subject = "weakref.proxy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: hash_proxy_raises (errors)."""
import weakref

class _Hashable:
    def __hash__(self):
        return 42

_h = _Hashable()
_p = weakref.proxy(_h)

_raised = False
try:
    hash(_p)
except TypeError:
    _raised = True
assert _raised, "hash_proxy_raises: expected TypeError"
print("hash_proxy_raises OK")
"###);
    assert_output(&out, r###"hash_proxy_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/weakref/proxy_int_raises.py`.
#[test]
fn test_gen_errors_std_libs_weakref_proxy_int_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "proxy_int_raises"
# subject = "weakref.proxy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: proxy_int_raises (errors)."""
import weakref

_raised = False
try:
    weakref.proxy(42)
except TypeError:
    _raised = True
assert _raised, "proxy_int_raises: expected TypeError"
print("proxy_int_raises OK")
"###);
    assert_output(&out, r###"proxy_int_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/weakref/ref_int_raises.py`.
#[test]
fn test_gen_errors_std_libs_weakref_ref_int_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "ref_int_raises"
# subject = "weakref.ref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: ref_int_raises (errors)."""
import weakref

_raised = False
try:
    weakref.ref(42)
except TypeError:
    _raised = True
assert _raised, "ref_int_raises: expected TypeError"
print("ref_int_raises OK")
"###);
    assert_output(&out, r###"ref_int_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/weakref/ref_reinit_bogus_args_raises.py`.
#[test]
fn test_gen_errors_std_libs_weakref_ref_reinit_bogus_args_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "ref_reinit_bogus_args_raises"
# subject = "weakref.ref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: ref_reinit_bogus_args_raises (errors)."""
import weakref

_r = weakref.ref(Exception)

_raised = False
try:
    _r.__init__(0, 0, 0, 0, 0)
except TypeError:
    _raised = True
assert _raised, "ref_reinit_bogus_args_raises: expected TypeError"
print("ref_reinit_bogus_args_raises OK")
"###);
    assert_output(&out, r###"ref_reinit_bogus_args_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/weakref/ref_str_raises.py`.
#[test]
fn test_gen_errors_std_libs_weakref_ref_str_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "ref_str_raises"
# subject = "weakref.ref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: ref_str_raises (errors)."""
import weakref

_raised = False
try:
    weakref.ref('hello')
except TypeError:
    _raised = True
assert _raised, "ref_str_raises: expected TypeError"
print("ref_str_raises OK")
"###);
    assert_output(&out, r###"ref_str_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/weakref/ref_tuple_raises.py`.
#[test]
fn test_gen_errors_std_libs_weakref_ref_tuple_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "ref_tuple_raises"
# subject = "weakref.ref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: ref_tuple_raises (errors)."""
import weakref

_raised = False
try:
    weakref.ref((1, 2, 3))
except TypeError:
    _raised = True
assert _raised, "ref_tuple_raises: expected TypeError"
print("ref_tuple_raises OK")
"###);
    assert_output(&out, r###"ref_tuple_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/weakref/weakmethod_plain_function_raises.py`.
#[test]
fn test_gen_errors_std_libs_weakref_weakmethod_plain_function_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "weakmethod_plain_function_raises"
# subject = "weakref.WeakMethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.WeakMethod: weakmethod_plain_function_raises (errors)."""
import weakref

def _meth():
    return 1

_raised = False
try:
    weakref.WeakMethod(_meth)
except TypeError:
    _raised = True
assert _raised, "weakmethod_plain_function_raises: expected TypeError"
print("weakmethod_plain_function_raises OK")
"###);
    assert_output(&out, r###"weakmethod_plain_function_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/weakref/weakvaluedictionary_int_value_raises.py`.
#[test]
fn test_gen_errors_std_libs_weakref_weakvaluedictionary_int_value_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "weakvaluedictionary_int_value_raises"
# subject = "weakref.WeakValueDictionary"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.WeakValueDictionary: weakvaluedictionary_int_value_raises (errors)."""
import weakref

_wvd = weakref.WeakValueDictionary()

_raised = False
try:
    _wvd.__setitem__('k', 42)
except TypeError:
    _raised = True
assert _raised, "weakvaluedictionary_int_value_raises: expected TypeError"
print("weakvaluedictionary_int_value_raises OK")
"###);
    assert_output(&out, r###"weakvaluedictionary_int_value_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/weakref/weakvaluedictionary_missing_key_raises.py`.
#[test]
fn test_gen_errors_std_libs_weakref_weakvaluedictionary_missing_key_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "weakvaluedictionary_missing_key_raises"
# subject = "weakref.WeakValueDictionary"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.WeakValueDictionary: weakvaluedictionary_missing_key_raises (errors)."""
import weakref

_wvd = weakref.WeakValueDictionary()

_raised = False
try:
    _wvd['missing']
except KeyError:
    _raised = True
assert _raised, "weakvaluedictionary_missing_key_raises: expected KeyError"
print("weakvaluedictionary_missing_key_raises OK")
"###);
    assert_output(&out, r###"weakvaluedictionary_missing_key_raises OK
"###);
}
