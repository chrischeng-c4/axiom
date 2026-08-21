# /// script
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
