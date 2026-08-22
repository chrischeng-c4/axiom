use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/tempfile/infer_return_type_mixes_str_bytes_raises.py`.
#[test]
fn test_gen_errors_std_libs_tempfile_infer_return_type_mixes_str_bytes_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "errors"
# case = "infer_return_type_mixes_str_bytes_raises"
# subject = "tempfile._infer_return_type"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile._infer_return_type: infer_return_type_mixes_str_bytes_raises (errors)."""
import tempfile

_raised = False
try:
    tempfile._infer_return_type('', b'')
except TypeError:
    _raised = True
assert _raised, "infer_return_type_mixes_str_bytes_raises: expected TypeError"
print("infer_return_type_mixes_str_bytes_raises OK")
"###);
    assert_output(&out, r###"infer_return_type_mixes_str_bytes_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tempfile/mkstemp_nonexistent_dir_raises.py`.
#[test]
fn test_gen_errors_std_libs_tempfile_mkstemp_nonexistent_dir_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "errors"
# case = "mkstemp_nonexistent_dir_raises"
# subject = "tempfile.mkstemp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.mkstemp: mkstemp_nonexistent_dir_raises (errors)."""
import tempfile

_raised = False
try:
    tempfile.mkstemp(dir='/nonexistent_dir_xyzzy')
except FileNotFoundError:
    _raised = True
assert _raised, "mkstemp_nonexistent_dir_raises: expected FileNotFoundError"
print("mkstemp_nonexistent_dir_raises OK")
"###);
    assert_output(&out, r###"mkstemp_nonexistent_dir_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tempfile/named_file_bad_mode_raises.py`.
#[test]
fn test_gen_errors_std_libs_tempfile_named_file_bad_mode_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "errors"
# case = "named_file_bad_mode_raises"
# subject = "tempfile.NamedTemporaryFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: named_file_bad_mode_raises (errors)."""
import tempfile

_raised = False
try:
    tempfile.NamedTemporaryFile(mode='Q')
except ValueError:
    _raised = True
assert _raised, "named_file_bad_mode_raises: expected ValueError"
print("named_file_bad_mode_raises OK")
"###);
    assert_output(&out, r###"named_file_bad_mode_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tempfile/read_closed_named_file_raises.py`.
#[test]
fn test_gen_errors_std_libs_tempfile_read_closed_named_file_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "errors"
# case = "read_closed_named_file_raises"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: reading a NamedTemporaryFile after it has been closed raises ValueError (the open w+b file reads fine; only the closed handle raises)"""
import os
import tempfile

f = tempfile.NamedTemporaryFile(delete=False)
f.write(b"data")
path = f.name
f.close()
_raised = False
try:
    f.read()
except ValueError:
    _raised = True
finally:
    try:
        os.unlink(path)
    except OSError:
        pass
assert _raised, "read_closed_named_file_raises: expected ValueError"
print("read_closed_named_file_raises OK")
"###);
    assert_output(&out, r###"read_closed_named_file_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tempfile/spooled_negative_max_size_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_tempfile_spooled_negative_max_size_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "errors"
# case = "spooled_negative_max_size_no_raise"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: SpooledTemporaryFile(max_size=-1) does NOT raise; a negative max_size just rolls the spool to disk on first write"""
import tempfile

spool = tempfile.SpooledTemporaryFile(max_size=-1)
spool.write(b"x" * 100)
assert spool._rolled is True, "negative max_size rolls to disk on first write"
spool.close()
print("spooled_negative_max_size_no_raise OK")
"###);
    assert_output(&out, r###"spooled_negative_max_size_no_raise OK
"###);
}
