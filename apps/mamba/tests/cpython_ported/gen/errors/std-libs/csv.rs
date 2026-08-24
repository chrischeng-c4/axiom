use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/csv/dictwriter_extra_field_raises.py`.
#[test]
fn test_gen_errors_std_libs_csv_dictwriter_extra_field_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "dictwriter_extra_field_raises"
# subject = "csv.DictWriter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.DictWriter: dictwriter_extra_field_raises (errors)."""
import csv

_raised = False
try:
    csv.DictWriter(__import__("io").StringIO(), fieldnames=["a", "b"], extrasaction="raise").writerow({"a": 1, "b": 2, "extra": 3})
except ValueError:
    _raised = True
assert _raised, "dictwriter_extra_field_raises: expected ValueError"
print("dictwriter_extra_field_raises OK")
"###);
    assert_output(&out, r###"dictwriter_extra_field_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/csv/field_too_large_raises.py`.
#[test]
fn test_gen_errors_std_libs_csv_field_too_large_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "field_too_large_raises"
# subject = "csv.field_size_limit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.field_size_limit: field_too_large_raises (errors)."""
import csv

_raised = False
try:
    csv.field_size_limit(10) and list(csv.reader(["a" * 50 + ",b"]))
except csv.Error:
    _raised = True
assert _raised, "field_too_large_raises: expected csv.Error"
print("field_too_large_raises OK")
"###);
    assert_output(&out, r###"field_too_large_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/csv/get_unknown_dialect_raises.py`.
#[test]
fn test_gen_errors_std_libs_csv_get_unknown_dialect_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "get_unknown_dialect_raises"
# subject = "csv.get_dialect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.get_dialect: get_unknown_dialect_raises (errors)."""
import csv

_raised = False
try:
    csv.get_dialect("nonesuch")
except csv.Error:
    _raised = True
assert _raised, "get_unknown_dialect_raises: expected csv.Error"
print("get_unknown_dialect_raises OK")
"###);
    assert_output(&out, r###"get_unknown_dialect_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/csv/incomplete_dialect_subclass_raises.py`.
#[test]
fn test_gen_errors_std_libs_csv_incomplete_dialect_subclass_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "incomplete_dialect_subclass_raises"
# subject = "csv.Dialect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.Dialect: incomplete_dialect_subclass_raises (errors)."""
import csv

_raised = False
try:
    type("_Incomplete", (csv.Dialect,), {"delimiter": "\\t"})()
except csv.Error:
    _raised = True
assert _raised, "incomplete_dialect_subclass_raises: expected csv.Error"
print("incomplete_dialect_subclass_raises OK")
"###);
    assert_output(&out, r###"incomplete_dialect_subclass_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/csv/unknown_dialect_raises.py`.
#[test]
fn test_gen_errors_std_libs_csv_unknown_dialect_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "unknown_dialect_raises"
# subject = "csv.reader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.reader: unknown_dialect_raises (errors)."""
import csv

_raised = False
try:
    csv.reader(["a,b"], dialect="no_such_dialect")
except csv.Error:
    _raised = True
assert _raised, "unknown_dialect_raises: expected csv.Error"
print("unknown_dialect_raises OK")
"###);
    assert_output(&out, r###"unknown_dialect_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/csv/unregister_unknown_dialect_raises.py`.
#[test]
fn test_gen_errors_std_libs_csv_unregister_unknown_dialect_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "unregister_unknown_dialect_raises"
# subject = "csv.unregister_dialect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.unregister_dialect: unregister_unknown_dialect_raises (errors)."""
import csv

_raised = False
try:
    csv.unregister_dialect("nonesuch")
except csv.Error:
    _raised = True
assert _raised, "unregister_unknown_dialect_raises: expected csv.Error"
print("unregister_unknown_dialect_raises OK")
"###);
    assert_output(&out, r###"unregister_unknown_dialect_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/csv/writerows_none_raises.py`.
#[test]
fn test_gen_errors_std_libs_csv_writerows_none_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "writerows_none_raises"
# subject = "csv.writer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.writer: writerows_none_raises (errors)."""
import csv

_raised = False
try:
    csv.writer([]).writerows(None)
except TypeError:
    _raised = True
assert _raised, "writerows_none_raises: expected TypeError"
print("writerows_none_raises OK")
"###);
    assert_output(&out, r###"writerows_none_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/csv/writerows_propagates_file_error.py`.
#[test]
fn test_gen_errors_std_libs_csv_writerows_propagates_file_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "writerows_propagates_file_error"
# subject = "csv.writer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.writer: writerows_propagates_file_error (errors)."""
import csv
class _BrokenFile:
    def write(self, buf):
        raise OSError('boom')

_raised = False
try:
    csv.writer(_BrokenFile()).writerows([['a']])
except OSError:
    _raised = True
assert _raised, "writerows_propagates_file_error: expected OSError"
print("writerows_propagates_file_error OK")
"###);
    assert_output(&out, r###"writerows_propagates_file_error OK
"###);
}
