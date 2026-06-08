# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_subprocess_constants_ops"
# subject = "cpython321.test_subprocess_constants_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_subprocess_constants_ops.py"
# status = "filled"
# ///
"""cpython321.test_subprocess_constants_ops: execute CPython 3.12 seed test_subprocess_constants_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `subprocess` stream-redirection
# sentinel constants.
# Surface: PIPE = -1, STDOUT = -2, DEVNULL = -3. These sentinel values
# are part of the documented subprocess API used to direct a child
# process's stdin/stdout/stderr.
# Companion to stub/test_subprocess.py — vendored unittest seed.
import subprocess
_ledger: list[int] = []
assert subprocess.PIPE == -1; _ledger.append(1)
assert subprocess.STDOUT == -2; _ledger.append(1)
assert subprocess.DEVNULL == -3; _ledger.append(1)
# All three sentinels are distinct
assert subprocess.PIPE != subprocess.STDOUT; _ledger.append(1)
assert subprocess.STDOUT != subprocess.DEVNULL; _ledger.append(1)
assert subprocess.PIPE != subprocess.DEVNULL; _ledger.append(1)
# All three are negative (so they cannot collide with a real file
# descriptor)
assert subprocess.PIPE < 0; _ledger.append(1)
assert subprocess.STDOUT < 0; _ledger.append(1)
assert subprocess.DEVNULL < 0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_subprocess_constants_ops {sum(_ledger)} asserts")
