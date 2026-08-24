use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/bdb/get_bpbynumber_non_numeric_raises.py`.
#[test]
fn test_gen_errors_std_libs_bdb_get_bpbynumber_non_numeric_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "errors"
# case = "get_bpbynumber_non_numeric_raises"
# subject = "bdb.Bdb.get_bpbynumber"
# kind = "mechanical"
# xfail = "mamba bdb stub: Bdb() is dict-like, no get_bpbynumber method (#1261)"
# mem_carveout = ""
# source = "Lib/test/test_bdb.py"
# status = "filled"
# ///
"""bdb.Bdb.get_bpbynumber: get_bpbynumber_non_numeric_raises (errors)."""
import bdb

_raised = False
try:
    bdb.Bdb().get_bpbynumber('not_a_number')
except ValueError:
    _raised = True
assert _raised, "get_bpbynumber_non_numeric_raises: expected ValueError"
print("get_bpbynumber_non_numeric_raises OK")
"###);
    assert_output(&out, r###"get_bpbynumber_non_numeric_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/bdb/get_bpbynumber_out_of_range_raises.py`.
#[test]
fn test_gen_errors_std_libs_bdb_get_bpbynumber_out_of_range_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "errors"
# case = "get_bpbynumber_out_of_range_raises"
# subject = "bdb.Bdb.get_bpbynumber"
# kind = "mechanical"
# xfail = "mamba bdb stub: Bdb() is dict-like, no get_bpbynumber method (#1261)"
# mem_carveout = ""
# source = "Lib/test/test_bdb.py"
# status = "filled"
# ///
"""bdb.Bdb.get_bpbynumber: get_bpbynumber_out_of_range_raises (errors)."""
import bdb

_raised = False
try:
    bdb.Bdb().get_bpbynumber(9999)
except ValueError:
    _raised = True
assert _raised, "get_bpbynumber_out_of_range_raises: expected ValueError"
print("get_bpbynumber_out_of_range_raises OK")
"###);
    assert_output(&out, r###"get_bpbynumber_out_of_range_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/bdb/runeval_syntax_error_raises.py`.
#[test]
fn test_gen_errors_std_libs_bdb_runeval_syntax_error_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "errors"
# case = "runeval_syntax_error_raises"
# subject = "bdb.Bdb.runeval"
# kind = "mechanical"
# xfail = "mamba bdb stub: Bdb() is dict-like, no runeval method (#1261)"
# mem_carveout = ""
# source = "Lib/test/test_bdb.py"
# status = "filled"
# ///
"""bdb.Bdb.runeval: runeval_syntax_error_raises (errors)."""
import bdb

_raised = False
try:
    bdb.Bdb().runeval('def 0bad():')
except SyntaxError:
    _raised = True
assert _raised, "runeval_syntax_error_raises: expected SyntaxError"
print("runeval_syntax_error_raises OK")
"###);
    assert_output(&out, r###"runeval_syntax_error_raises OK
"###);
}
