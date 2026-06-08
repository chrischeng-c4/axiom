# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "run_input_feeds_stdin"
# subject = "subprocess.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.run: the input= keyword feeds data to the child's stdin and the child reads it back"""
import subprocess
import sys

_r = subprocess.run(
    [sys.executable, "-c", "import sys; print(sys.stdin.read().strip())"],
    input="fed_data\n", text=True, capture_output=True,
)
assert _r.stdout.strip() == "fed_data", f"stdin input = {_r.stdout!r}"
print("run_input_feeds_stdin OK")
