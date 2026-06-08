# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "popen_stdout_only_pipe_stderr_none"
# subject = "subprocess.Popen"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.Popen: with only stdout wired to PIPE, communicate() returns the stdout bytes and the unwired stderr slot as None"""
import subprocess
import sys

p = subprocess.Popen(
    [sys.executable, "-c", "import sys; sys.stdout.write('pineapple')"],
    stdout=subprocess.PIPE,
)
out, err = p.communicate()
assert out == b"pineapple", f"stdout-only out = {out!r}"
assert err is None, f"stdout-only err = {err!r}"
print("popen_stdout_only_pipe_stderr_none OK")
