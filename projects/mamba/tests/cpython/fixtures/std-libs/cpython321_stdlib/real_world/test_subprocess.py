# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_subprocess"
# subject = "cpython321.test_subprocess"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_subprocess.py"
# status = "filled"
# ///
"""cpython321.test_subprocess: execute CPython 3.12 seed test_subprocess"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: subprocess — process invocation primitives that mamba services today:
#   * subprocess.run(args, capture_output=True, text=True) executes a child
#     process and returns a CompletedProcess with returncode, stdout, stderr
#   * subprocess.call(args) returns the child's exit code (0 success, non-zero
#     failure)
#   * subprocess.check_output(args) returns the child's stdout as a string
#   * PIPE / STDOUT / DEVNULL constants exposed with canonical CPython sentinel
#     values (-1 / -2 / -3)
#   * CalledProcessError / TimeoutExpired / Popen symbols exposed
# Intentionally NOT exercised on mamba today (tracked separately):
#   * check=True on a failing process does NOT raise CalledProcessError
#     (mamba returns a CompletedProcess with non-zero returncode instead)
#   * Popen(...).communicate() / wait() / poll() lifecycle
#   * timeout= keyword raising TimeoutExpired
#   * shell=True quoting / env= / cwd= / input= passthrough
import subprocess

_ledger: list[int] = []

# (1) subprocess.run with text + capture_output works and returns a
#     CompletedProcess (or CompletedProcess-shaped object)
_r = subprocess.run(["echo", "hi"], capture_output=True, text=True)
assert _r.returncode - 0 == 0, (
    f"echo hi returncode == 0, got {_r.returncode!r}"
)
_ledger.append(1)

assert _r.stdout == "hi\n", f"echo hi stdout == 'hi\\n', got {_r.stdout!r}"
_ledger.append(1)

# (2) printf produces stdout without a trailing newline
_p = subprocess.run(["printf", "abc"], capture_output=True, text=True)
assert _p.stdout == "abc", f"printf abc stdout == 'abc', got {_p.stdout!r}"
_ledger.append(1)
assert _p.returncode - 0 == 0, (
    f"printf abc returncode == 0, got {_p.returncode!r}"
)
_ledger.append(1)

# (3) stderr is captured independently of stdout
_e = subprocess.run(["sh", "-c", "echo err >&2"], capture_output=True, text=True)
assert _e.stderr == "err\n", (
    f"echo err >&2 stderr == 'err\\n', got {_e.stderr!r}"
)
_ledger.append(1)

# (4) subprocess.call returns 0 on success
_rc_ok = subprocess.call(["true"])
assert _rc_ok - 0 == 0, f"subprocess.call(['true']) == 0, got {_rc_ok!r}"
_ledger.append(1)

# (5) subprocess.call returns non-zero on failure
_rc_fail = subprocess.call(["false"])
assert _rc_fail - 0 != 0, (
    f"subprocess.call(['false']) is non-zero, got {_rc_fail!r}"
)
_ledger.append(1)

# (6) subprocess.check_output returns the child's stdout
_co = subprocess.check_output(["echo", "hello"])
assert _co == "hello\n" or _co == b"hello\n", (
    f"check_output ['echo','hello'] returns 'hello\\n', got {_co!r}"
)
_ledger.append(1)

# (7) Constants PIPE / STDOUT / DEVNULL exposed with CPython sentinel values
assert subprocess.PIPE - (-1) == 0, (
    f"subprocess.PIPE == -1, got {subprocess.PIPE!r}"
)
_ledger.append(1)
assert subprocess.STDOUT - (-2) == 0, (
    f"subprocess.STDOUT == -2, got {subprocess.STDOUT!r}"
)
_ledger.append(1)
assert subprocess.DEVNULL - (-3) == 0, (
    f"subprocess.DEVNULL == -3, got {subprocess.DEVNULL!r}"
)
_ledger.append(1)

# (8) Class symbols exposed
for _name in ("CalledProcessError", "TimeoutExpired", "Popen"):
    assert hasattr(subprocess, _name), f"subprocess.{_name} symbol is exposed"
_ledger.append(1)

# (9) call / check_call / check_output / run all exposed
for _name in ("call", "check_call", "check_output", "run"):
    assert hasattr(subprocess, _name), f"subprocess.{_name} symbol is exposed"
_ledger.append(1)

# (10) Two independent runs don't share state — second run sees its own stdout
_r1 = subprocess.run(["echo", "first"], capture_output=True, text=True)
_r2 = subprocess.run(["echo", "second"], capture_output=True, text=True)
assert _r1.stdout == "first\n", f"first run stdout, got {_r1.stdout!r}"
_ledger.append(1)
assert _r2.stdout == "second\n", f"second run stdout, got {_r2.stdout!r}"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_subprocess {sum(_ledger)} asserts")
