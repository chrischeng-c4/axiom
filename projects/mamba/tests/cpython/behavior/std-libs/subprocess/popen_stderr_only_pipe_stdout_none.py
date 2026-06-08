# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "popen_stderr_only_pipe_stdout_none"
# subject = "subprocess.Popen"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.Popen: with only stderr wired to PIPE, communicate() returns the stderr bytes and the unwired stdout slot as None"""
import subprocess
import sys

p = subprocess.Popen(
    [sys.executable, "-c", "import sys; sys.stderr.write('strawberry')"],
    stderr=subprocess.PIPE,
)
out, err = p.communicate()
assert out is None, f"stderr-only out = {out!r}"
assert err == b"strawberry", f"stderr-only err = {err!r}"
print("popen_stderr_only_pipe_stdout_none OK")
