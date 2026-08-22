use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/subprocess/all_exports_public_api.py`.
#[test]
fn test_gen_behavior_std_libs_subprocess_all_exports_public_api() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "all_exports_public_api"
# subject = "subprocess.__all__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.__all__: __all__ exports run and Popen but deliberately omits the low-level helper list2cmdline"""
import subprocess

assert "run" in subprocess.__all__, "run in __all__"
assert "Popen" in subprocess.__all__, "Popen in __all__"
assert "list2cmdline" not in subprocess.__all__, "list2cmdline excluded"
print("all_exports_public_api OK")
"###);
    assert_output(&out, r###"all_exports_public_api OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/subprocess/getoutput_returns_trimmed_stdout.py`.
#[test]
fn test_gen_behavior_std_libs_subprocess_getoutput_returns_trimmed_stdout() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "getoutput_returns_trimmed_stdout"
# subject = "subprocess.getoutput"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.getoutput: subprocess.getoutput runs a shell command and returns its trimmed stdout text"""
import subprocess

assert subprocess.getoutput("echo xyzzy") == "xyzzy", "getoutput"
print("getoutput_returns_trimmed_stdout OK")
"###);
    assert_output(&out, r###"getoutput_returns_trimmed_stdout OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/subprocess/getstatusoutput_pairs_status_and_output.py`.
#[test]
fn test_gen_behavior_std_libs_subprocess_getstatusoutput_pairs_status_and_output() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "getstatusoutput_pairs_status_and_output"
# subject = "subprocess.getstatusoutput"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.getstatusoutput: subprocess.getstatusoutput returns (exit_status, output): zero status on success and a non-zero status for a failing shell command"""
import subprocess

status, output = subprocess.getstatusoutput("echo xyzzy")
assert (status, output) == (0, "xyzzy"), f"getstatusoutput = {(status, output)!r}"
status, _ = subprocess.getstatusoutput("exit 5")
assert status != 0, f"failing status = {status!r}"
print("getstatusoutput_pairs_status_and_output OK")
"###);
    assert_output(&out, r###"getstatusoutput_pairs_status_and_output OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/subprocess/list2cmdline_quotes_args.py`.
#[test]
fn test_gen_behavior_std_libs_subprocess_list2cmdline_quotes_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "list2cmdline_quotes_args"
# subject = "subprocess.list2cmdline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.list2cmdline: list2cmdline quotes arguments containing spaces and renders an empty argument as a quoted empty string"""
import subprocess

assert subprocess.list2cmdline(["a b c", "d", "e"]) == '"a b c" d e', "quote spaces"
assert subprocess.list2cmdline(["ab", ""]) == 'ab ""', "quote empty arg"
print("list2cmdline_quotes_args OK")
"###);
    assert_output(&out, r###"list2cmdline_quotes_args OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/subprocess/popen_completedprocess_generic_alias.py`.
#[test]
fn test_gen_behavior_std_libs_subprocess_popen_completedprocess_generic_alias() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "popen_completedprocess_generic_alias"
# subject = "subprocess.Popen"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.Popen: Popen[bytes] and CompletedProcess[str] subscription each yield a types.GenericAlias"""
import subprocess
import types

assert isinstance(subprocess.Popen[bytes], types.GenericAlias), "Popen[bytes]"
assert isinstance(
    subprocess.CompletedProcess[str], types.GenericAlias
), "CompletedProcess[str]"
print("popen_completedprocess_generic_alias OK")
"###);
    assert_output(&out, r###"popen_completedprocess_generic_alias OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/subprocess/popen_devnull_leaves_file_objects_none.py`.
#[test]
fn test_gen_behavior_std_libs_subprocess_popen_devnull_leaves_file_objects_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "popen_devnull_leaves_file_objects_none"
# subject = "subprocess.Popen"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.Popen: DEVNULL for stdin/stdout leaves the corresponding Popen.stdin / Popen.stdout file objects as None"""
import subprocess
import sys

p = subprocess.Popen(
    [sys.executable, "-c", "pass"],
    stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
)
p.wait()
assert p.stdin is None, f"devnull stdin = {p.stdin!r}"
assert p.stdout is None, f"devnull stdout = {p.stdout!r}"
print("popen_devnull_leaves_file_objects_none OK")
"###);
    assert_output(&out, r###"popen_devnull_leaves_file_objects_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/subprocess/popen_pipe_streams_are_buffered.py`.
#[test]
fn test_gen_behavior_std_libs_subprocess_popen_pipe_streams_are_buffered() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "popen_pipe_streams_are_buffered"
# subject = "subprocess.Popen"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.Popen: default PIPE streams are io.BufferedIOBase instances (binary, no text mode) for stdin/stdout/stderr"""
import io
import subprocess
import sys

p = subprocess.Popen(
    [sys.executable, "-c", "pass"],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
)
assert isinstance(p.stdin, io.BufferedIOBase), f"stdin type = {type(p.stdin)!r}"
assert isinstance(p.stdout, io.BufferedIOBase), f"stdout type = {type(p.stdout)!r}"
assert isinstance(p.stderr, io.BufferedIOBase), f"stderr type = {type(p.stderr)!r}"
p.stdin.close()
p.stdout.close()
p.stderr.close()
p.wait()
print("popen_pipe_streams_are_buffered OK")
"###);
    assert_output(&out, r###"popen_pipe_streams_are_buffered OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/subprocess/run_returns_completedprocess.py`.
#[test]
fn test_gen_behavior_std_libs_subprocess_run_returns_completedprocess() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "run_returns_completedprocess"
# subject = "subprocess.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.run: subprocess.run(capture_output=True, text=True) returns a CompletedProcess with returncode 0, captured stdout, and accessible str stderr for a simple echo"""
import subprocess

_r = subprocess.run(["echo", "hello"], capture_output=True, text=True)
assert isinstance(_r, subprocess.CompletedProcess), f"run type = {type(_r)!r}"
assert _r.returncode == 0, f"echo returncode = {_r.returncode!r}"
assert "hello" in _r.stdout, f"stdout = {_r.stdout!r}"
assert isinstance(_r.stderr, str), f"stderr type = {type(_r.stderr)!r}"
assert _r.args == ["echo", "hello"], f"args = {_r.args!r}"
print("run_returns_completedprocess OK")
"###);
    assert_output(&out, r###"run_returns_completedprocess OK
"###);
}
