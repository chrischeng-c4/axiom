use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/tarfile/compresslevel_bz2_zero_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_tarfile_compresslevel_bz2_zero_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "compresslevel_bz2_zero_raises_valueerror"
# subject = "tarfile.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile.open: compresslevel_bz2_zero_raises_valueerror (errors)."""
import tarfile
import io

_raised = False
try:
    tarfile.open(fileobj=io.BytesIO(), mode='w:bz2', compresslevel=0)
except ValueError:
    _raised = True
assert _raised, "compresslevel_bz2_zero_raises_valueerror: expected ValueError"
print("compresslevel_bz2_zero_raises_valueerror OK")
"###);
    assert_output(&out, r###"compresslevel_bz2_zero_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tarfile/compresslevel_plain_mode_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_tarfile_compresslevel_plain_mode_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "compresslevel_plain_mode_raises_typeerror"
# subject = "tarfile.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile.open: compresslevel_plain_mode_raises_typeerror (errors)."""
import tarfile
import io

_raised = False
try:
    tarfile.open(fileobj=io.BytesIO(), mode='w:', compresslevel=5)
except TypeError:
    _raised = True
assert _raised, "compresslevel_plain_mode_raises_typeerror: expected TypeError"
print("compresslevel_plain_mode_raises_typeerror OK")
"###);
    assert_output(&out, r###"compresslevel_plain_mode_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tarfile/data_filter_parent_traversal_raises.py`.
#[test]
fn test_gen_errors_std_libs_tarfile_data_filter_parent_traversal_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "data_filter_parent_traversal_raises"
# subject = "tarfile.data_filter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.data_filter: data_filter_parent_traversal_raises (errors)."""
import tarfile
_esc = tarfile.TarInfo('../escape.txt')
_esc.size = 0

_raised = False
try:
    tarfile.data_filter(_esc, 'dest')
except tarfile.OutsideDestinationError:
    _raised = True
assert _raised, "data_filter_parent_traversal_raises: expected tarfile.OutsideDestinationError"
print("data_filter_parent_traversal_raises OK")
"###);
    assert_output(&out, r###"data_filter_parent_traversal_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tarfile/extractall_bad_filter_name_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_tarfile_extractall_bad_filter_name_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "extractall_bad_filter_name_raises_valueerror"
# subject = "tarfile.TarFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: extractall_bad_filter_name_raises_valueerror (errors)."""
import tarfile
import io
_buf = io.BytesIO()
tarfile.open(fileobj=_buf, mode='w').close()
_buf.seek(0)
_tf = tarfile.open(fileobj=_buf, mode='r')

_raised = False
try:
    _tf.extractall('dest', filter='nope')
except ValueError:
    _raised = True
assert _raised, "extractall_bad_filter_name_raises_valueerror: expected ValueError"
print("extractall_bad_filter_name_raises_valueerror OK")
"###);
    assert_output(&out, r###"extractall_bad_filter_name_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tarfile/gnu_uid_overflow_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_tarfile_gnu_uid_overflow_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "gnu_uid_overflow_raises_valueerror"
# subject = "tarfile.TarInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarInfo: gnu_uid_overflow_raises_valueerror (errors)."""
import tarfile
_ti = tarfile.TarInfo('name')
_ti.uid = 72057594037927936

_raised = False
try:
    _ti.tobuf(tarfile.GNU_FORMAT)
except ValueError:
    _raised = True
assert _raised, "gnu_uid_overflow_raises_valueerror: expected ValueError"
print("gnu_uid_overflow_raises_valueerror OK")
"###);
    assert_output(&out, r###"gnu_uid_overflow_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tarfile/itn_negative_ustar_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_tarfile_itn_negative_ustar_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "itn_negative_ustar_raises_valueerror"
# subject = "tarfile.itn"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.itn: itn_negative_ustar_raises_valueerror (errors)."""
import tarfile

_raised = False
try:
    tarfile.itn(-1, 8, tarfile.USTAR_FORMAT)
except ValueError:
    _raised = True
assert _raised, "itn_negative_ustar_raises_valueerror: expected ValueError"
print("itn_negative_ustar_raises_valueerror OK")
"###);
    assert_output(&out, r###"itn_negative_ustar_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tarfile/itn_too_big_ustar_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_tarfile_itn_too_big_ustar_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "itn_too_big_ustar_raises_valueerror"
# subject = "tarfile.itn"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.itn: itn_too_big_ustar_raises_valueerror (errors)."""
import tarfile

_raised = False
try:
    tarfile.itn(2097152, 8, tarfile.USTAR_FORMAT)
except ValueError:
    _raised = True
assert _raised, "itn_too_big_ustar_raises_valueerror: expected ValueError"
print("itn_too_big_ustar_raises_valueerror OK")
"###);
    assert_output(&out, r###"itn_too_big_ustar_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tarfile/open_bad_mode_raises.py`.
#[test]
fn test_gen_errors_std_libs_tarfile_open_bad_mode_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "open_bad_mode_raises"
# subject = "tarfile.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile.open: open_bad_mode_raises (errors)."""
import tarfile
import io

_raised = False
try:
    tarfile.open(fileobj=io.BytesIO(), mode='X')
except (ValueError, tarfile.CompressionError):
    _raised = True
assert _raised, "open_bad_mode_raises: expected (ValueError, tarfile.CompressionError)"
print("open_bad_mode_raises OK")
"###);
    assert_output(&out, r###"open_bad_mode_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tarfile/open_missing_file_raises_filenotfound.py`.
#[test]
fn test_gen_errors_std_libs_tarfile_open_missing_file_raises_filenotfound() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "open_missing_file_raises_filenotfound"
# subject = "tarfile.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile.open: open_missing_file_raises_filenotfound (errors)."""
import tarfile

_raised = False
try:
    tarfile.open('/no/such/file.tar')
except FileNotFoundError:
    _raised = True
assert _raised, "open_missing_file_raises_filenotfound: expected FileNotFoundError"
print("open_missing_file_raises_filenotfound OK")
"###);
    assert_output(&out, r###"open_missing_file_raises_filenotfound OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tarfile/open_non_tar_raises_readerror.py`.
#[test]
fn test_gen_errors_std_libs_tarfile_open_non_tar_raises_readerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "open_non_tar_raises_readerror"
# subject = "tarfile.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.open: open_non_tar_raises_readerror (errors)."""
import tarfile
import io

_raised = False
try:
    tarfile.open(fileobj=io.BytesIO(b'not a tar file'))
except tarfile.ReadError:
    _raised = True
assert _raised, "open_non_tar_raises_readerror: expected tarfile.ReadError"
print("open_non_tar_raises_readerror OK")
"###);
    assert_output(&out, r###"open_non_tar_raises_readerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tarfile/ustar_name_too_long_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_tarfile_ustar_name_too_long_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "ustar_name_too_long_raises_valueerror"
# subject = "tarfile.TarInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarInfo: ustar_name_too_long_raises_valueerror (errors)."""
import tarfile

_raised = False
try:
    tarfile.TarInfo('0123456789' * 10 + '0').tobuf(tarfile.USTAR_FORMAT)
except ValueError:
    _raised = True
assert _raised, "ustar_name_too_long_raises_valueerror: expected ValueError"
print("ustar_name_too_long_raises_valueerror OK")
"###);
    assert_output(&out, r###"ustar_name_too_long_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tarfile/ustar_uid_overflow_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_tarfile_ustar_uid_overflow_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "ustar_uid_overflow_raises_valueerror"
# subject = "tarfile.TarInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarInfo: ustar_uid_overflow_raises_valueerror (errors)."""
import tarfile
_ti = tarfile.TarInfo('name')
_ti.uid = 2097152

_raised = False
try:
    _ti.tobuf(tarfile.USTAR_FORMAT)
except ValueError:
    _raised = True
assert _raised, "ustar_uid_overflow_raises_valueerror: expected ValueError"
print("ustar_uid_overflow_raises_valueerror OK")
"###);
    assert_output(&out, r###"ustar_uid_overflow_raises_valueerror OK
"###);
}
