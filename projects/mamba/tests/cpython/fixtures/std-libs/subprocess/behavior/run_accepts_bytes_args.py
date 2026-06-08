# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "run_accepts_bytes_args"
# subject = "subprocess.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.run: a bytes executable path and bytes argv entries are accepted; the child exit code propagates to returncode"""
import os
import subprocess
import sys

_r = subprocess.run([os.fsencode(sys.executable), "-c", b"import sys; sys.exit(57)"])
assert _r.returncode == 57, f"bytes args rc = {_r.returncode!r}"
print("run_accepts_bytes_args OK")
