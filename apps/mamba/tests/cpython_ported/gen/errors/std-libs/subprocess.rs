use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/subprocess/missing_command_raises_filenotfound.py`.
#[test]
fn test_gen_errors_std_libs_subprocess_missing_command_raises_filenotfound() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "errors"
# case = "missing_command_raises_filenotfound"
# subject = "subprocess.run"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.run: missing_command_raises_filenotfound (errors)."""
import subprocess

_raised = False
try:
    subprocess.run(['definitely_not_a_real_command_xyzzy'], capture_output=True)
except FileNotFoundError:
    _raised = True
assert _raised, "missing_command_raises_filenotfound: expected FileNotFoundError"
print("missing_command_raises_filenotfound OK")
"###);
    assert_output(&out, r###"missing_command_raises_filenotfound OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/subprocess/nul_in_argv_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_subprocess_nul_in_argv_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "errors"
# case = "nul_in_argv_raises_valueerror"
# subject = "subprocess.Popen"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.Popen: nul_in_argv_raises_valueerror (errors)."""
import subprocess
import sys

_raised = False
try:
    subprocess.Popen([sys.executable, '-c', 'pass#\x00'])
except ValueError:
    _raised = True
assert _raised, "nul_in_argv_raises_valueerror: expected ValueError"
print("nul_in_argv_raises_valueerror OK")
"###);
    assert_output(&out, r###"nul_in_argv_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/subprocess/nul_in_env_name_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_subprocess_nul_in_env_name_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "errors"
# case = "nul_in_env_name_raises_valueerror"
# subject = "subprocess.Popen"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.Popen: nul_in_env_name_raises_valueerror (errors)."""
import os
import subprocess
import sys

_raised = False
try:
    subprocess.Popen([sys.executable, '-c', 'pass'], env={**os.environ, 'FRUIT\x00VEGETABLE': 'cabbage'})
except ValueError:
    _raised = True
assert _raised, "nul_in_env_name_raises_valueerror: expected ValueError"
print("nul_in_env_name_raises_valueerror OK")
"###);
    assert_output(&out, r###"nul_in_env_name_raises_valueerror OK
"###);
}
