# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "popen_manual_stdin_write_and_idempotent_wait"
# subject = "subprocess.Popen"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.Popen: manually writing+closing stdin feeds the child and wait() is idempotent (second wait() returns the same returncode)"""
import subprocess
import sys

p = subprocess.Popen(
    [sys.executable, "-c", "import sys; sys.exit(sys.stdin.read() == 'pear')"],
    stdin=subprocess.PIPE,
)
p.stdin.write(b"pear")
p.stdin.close()
assert p.wait() == 1, f"first wait = {p.returncode!r}"
assert p.wait() == 1, "wait() is idempotent"
print("popen_manual_stdin_write_and_idempotent_wait OK")
