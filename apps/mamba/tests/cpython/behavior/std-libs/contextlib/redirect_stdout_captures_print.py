# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "redirect_stdout_captures_print"
# subject = "contextlib.redirect_stdout"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.redirect_stdout: redirect_stdout(buf) routes print() output into buf for the duration of the with-block and yields buf as the enter result"""
import contextlib
import io

buf = io.StringIO()
with contextlib.redirect_stdout(buf) as entered:
    assert entered is buf, "redirect_stdout yields the target stream"
    print("captured")
# Outside the block stdout is restored.
assert buf.getvalue() == "captured\n", f"redirect = {buf.getvalue()!r}"

print("redirect_stdout_captures_print OK")
