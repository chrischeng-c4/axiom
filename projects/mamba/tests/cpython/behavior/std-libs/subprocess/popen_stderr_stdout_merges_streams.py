# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "popen_stderr_stdout_merges_streams"
# subject = "subprocess.Popen"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.Popen: stderr=STDOUT merges the child's stderr into its stdout stream; communicate() returns the merged bytes on stdout and None for stderr"""
import subprocess
import sys

p = subprocess.Popen(
    [sys.executable, "-c", "import sys; sys.stderr.write('42')"],
    stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
)
out, err = p.communicate()
assert out == b"42", f"merged out = {out!r}"
assert err is None, f"merged err = {err!r}"
print("popen_stderr_stdout_merges_streams OK")
