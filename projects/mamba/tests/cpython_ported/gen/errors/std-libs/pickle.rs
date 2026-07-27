use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/pickle/bad_protocol_raises.py`.
#[test]
fn test_gen_errors_std_libs_pickle_bad_protocol_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "errors"
# case = "bad_protocol_raises"
# subject = "pickle.dumps"
# kind = "mechanical"
# xfail = "pickle shim ignores the protocol kwarg and never validates it (src/runtime/stdlib/pickle_mod.rs:318)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.dumps: bad_protocol_raises (errors)."""
import pickle

_raised = False
try:
    pickle.dumps('hi', protocol=99)
except ValueError:
    _raised = True
assert _raised, "bad_protocol_raises: expected ValueError"
print("bad_protocol_raises OK")
"###);
    assert_output(&out, r###"bad_protocol_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pickle/garbage_stream_raises.py`.
#[test]
fn test_gen_errors_std_libs_pickle_garbage_stream_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "errors"
# case = "garbage_stream_raises"
# subject = "pickle.loads"
# kind = "mechanical"
# xfail = "pickle shim returns None on bad input, never raises UnpicklingError (src/runtime/stdlib/pickle_mod.rs:324)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: garbage_stream_raises (errors)."""
import pickle

_raised = False
try:
    pickle.loads(b'not_a_pickle_stream')
except pickle.UnpicklingError:
    _raised = True
assert _raised, "garbage_stream_raises: expected pickle.UnpicklingError"
print("garbage_stream_raises OK")
"###);
    assert_output(&out, r###"garbage_stream_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pickle/generator_raises.py`.
#[test]
fn test_gen_errors_std_libs_pickle_generator_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "errors"
# case = "generator_raises"
# subject = "pickle.dumps"
# kind = "mechanical"
# xfail = "pickle shim serializes unsupported objects to the 'N' (None) sentinel instead of raising (src/runtime/stdlib/pickle_mod.rs:220)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.dumps: generator_raises (errors)."""
import pickle

_raised = False
try:
    pickle.dumps(i for i in range(3))
except (TypeError, pickle.PicklingError):
    _raised = True
assert _raised, "generator_raises: expected (TypeError, pickle.PicklingError)"
print("generator_raises OK")
"###);
    assert_output(&out, r###"generator_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pickle/lambda_raises.py`.
#[test]
fn test_gen_errors_std_libs_pickle_lambda_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "errors"
# case = "lambda_raises"
# subject = "pickle.dumps"
# kind = "mechanical"
# xfail = "pickle shim serializes unsupported objects to the 'N' (None) sentinel instead of raising PicklingError (src/runtime/stdlib/pickle_mod.rs:220)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.dumps: lambda_raises (errors)."""
import pickle

_raised = False
try:
    pickle.dumps(lambda x: x)
except (pickle.PicklingError, AttributeError):
    _raised = True
assert _raised, "lambda_raises: expected (pickle.PicklingError, AttributeError)"
print("lambda_raises OK")
"###);
    assert_output(&out, r###"lambda_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pickle/truncated_stream_raises.py`.
#[test]
fn test_gen_errors_std_libs_pickle_truncated_stream_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "errors"
# case = "truncated_stream_raises"
# subject = "pickle.loads"
# kind = "mechanical"
# xfail = "pickle shim returns None on bad input, never raises UnpicklingError (src/runtime/stdlib/pickle_mod.rs:324)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: truncated_stream_raises (errors)."""
import pickle

_raised = False
try:
    pickle.loads(b'\x80')
except pickle.UnpicklingError:
    _raised = True
assert _raised, "truncated_stream_raises: expected pickle.UnpicklingError"
print("truncated_stream_raises OK")
"###);
    assert_output(&out, r###"truncated_stream_raises OK
"###);
}
