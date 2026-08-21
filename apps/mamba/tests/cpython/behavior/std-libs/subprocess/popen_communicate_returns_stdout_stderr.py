# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "popen_communicate_returns_stdout_stderr"
# subject = "subprocess.Popen"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.Popen: Popen(stdout=PIPE, stderr=PIPE).communicate() returns the child's (stdout, stderr) as bytes and records returncode 0"""
import subprocess
import sys

p = subprocess.Popen(
    [sys.executable, "-c", "print('popen_out')"],
    stdout=subprocess.PIPE, stderr=subprocess.PIPE,
)
_stdout, _stderr = p.communicate()
assert b"popen_out" in _stdout, f"Popen stdout = {_stdout!r}"
assert _stderr == b"", f"Popen stderr = {_stderr!r}"
assert p.returncode == 0, f"Popen returncode = {p.returncode!r}"
print("popen_communicate_returns_stdout_stderr OK")
