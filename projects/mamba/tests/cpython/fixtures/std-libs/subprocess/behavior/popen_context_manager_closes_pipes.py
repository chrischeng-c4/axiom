# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "popen_context_manager_closes_pipes"
# subject = "subprocess.Popen"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.Popen: using Popen as a context manager closes its pipes on exit and records the child's exit code in returncode"""
import subprocess
import sys

with subprocess.Popen(
    [sys.executable, "-c", "import sys; sys.stdout.write('ctx'); sys.exit(100)"],
    stdout=subprocess.PIPE,
) as proc:
    assert proc.stdout.read() == b"ctx", "ctx stdout"
assert proc.stdout.closed, "stdout closed after with-block"
assert proc.returncode == 100, f"ctx returncode = {proc.returncode!r}"
print("popen_context_manager_closes_pipes OK")
