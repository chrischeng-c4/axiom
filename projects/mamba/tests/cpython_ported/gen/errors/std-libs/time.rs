use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/time/asctime_int_arg_raises.py`.
#[test]
fn test_gen_errors_std_libs_time_asctime_int_arg_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "errors"
# case = "asctime_int_arg_raises"
# subject = "time.asctime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.asctime: asctime_int_arg_raises (errors)."""
import time

_raised = False
try:
    time.asctime(123)
except TypeError:
    _raised = True
assert _raised, "asctime_int_arg_raises: expected TypeError"
print("asctime_int_arg_raises OK")
"###);
    assert_output(&out, r###"asctime_int_arg_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/time/clock_gettime_bad_id_raises.py`.
#[test]
fn test_gen_errors_std_libs_time_clock_gettime_bad_id_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "errors"
# case = "clock_gettime_bad_id_raises"
# subject = "time.clock_gettime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.clock_gettime: clock_gettime_bad_id_raises (errors)."""
import time

_raised = False
try:
    time.clock_gettime(999999)
except OSError:
    _raised = True
assert _raised, "clock_gettime_bad_id_raises: expected OSError"
print("clock_gettime_bad_id_raises OK")
"###);
    assert_output(&out, r###"clock_gettime_bad_id_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/time/insane_timestamp_overflows.py`.
#[test]
fn test_gen_errors_std_libs_time_insane_timestamp_overflows() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "errors"
# case = "insane_timestamp_overflows"
# subject = "time.gmtime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.gmtime: an out-of-range timestamp (1e200) raises OverflowError (not a garbage struct_time) across ctime, gmtime, and localtime"""
import time

for _fn in (time.ctime, time.gmtime, time.localtime):
    _raised = False
    try:
        _fn(1e200)
    except OverflowError:
        _raised = True
    assert _raised, f"{_fn.__name__}(1e200): expected OverflowError"
print("insane_timestamp_overflows OK")
"###);
    assert_output(&out, r###"insane_timestamp_overflows OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/time/mktime_short_tuple_raises.py`.
#[test]
fn test_gen_errors_std_libs_time_mktime_short_tuple_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "errors"
# case = "mktime_short_tuple_raises"
# subject = "time.mktime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.mktime: mktime_short_tuple_raises (errors)."""
import time

_raised = False
try:
    time.mktime((2024, 1, 1))
except TypeError:
    _raised = True
assert _raised, "mktime_short_tuple_raises: expected TypeError"
print("mktime_short_tuple_raises OK")
"###);
    assert_output(&out, r###"mktime_short_tuple_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/time/sleep_negative_raises.py`.
#[test]
fn test_gen_errors_std_libs_time_sleep_negative_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "errors"
# case = "sleep_negative_raises"
# subject = "time.sleep"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.sleep: sleep_negative_raises (errors)."""
import time

_raised = False
try:
    time.sleep(-1)
except ValueError:
    _raised = True
assert _raised, "sleep_negative_raises: expected ValueError"
print("sleep_negative_raises OK")
"###);
    assert_output(&out, r###"sleep_negative_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/time/strptime_format_mismatch_raises.py`.
#[test]
fn test_gen_errors_std_libs_time_strptime_format_mismatch_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "errors"
# case = "strptime_format_mismatch_raises"
# subject = "time.strptime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.strptime: strptime_format_mismatch_raises (errors)."""
import time

_raised = False
try:
    time.strptime('not_a_date', '%Y-%m-%d')
except ValueError:
    _raised = True
assert _raised, "strptime_format_mismatch_raises: expected ValueError"
print("strptime_format_mismatch_raises OK")
"###);
    assert_output(&out, r###"strptime_format_mismatch_raises OK
"###);
}
