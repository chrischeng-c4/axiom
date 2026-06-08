# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "redirect_stderr_captures_stderr"
# subject = "contextlib.redirect_stderr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.redirect_stderr: redirect_stderr(buf) routes writes to sys.stderr into buf for the duration of the with-block"""
import contextlib
import io
import sys

buf = io.StringIO()
with contextlib.redirect_stderr(buf):
    print("error output", file=sys.stderr)
assert "error output" in buf.getvalue(), f"redirect_stderr = {buf.getvalue()!r}"

print("redirect_stderr_captures_stderr OK")
