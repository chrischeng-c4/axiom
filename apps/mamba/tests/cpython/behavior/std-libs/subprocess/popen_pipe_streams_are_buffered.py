# /// script
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
