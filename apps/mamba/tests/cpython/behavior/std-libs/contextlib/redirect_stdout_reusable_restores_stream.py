# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "redirect_stdout_reusable_restores_stream"
# subject = "contextlib.redirect_stdout"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.redirect_stdout: a single redirect_stdout(buf) instance is reusable: re-entering keeps writing to the same target and sys.stdout is restored after each block"""
import contextlib
import io
import sys

buf = io.StringIO()
redir = contextlib.redirect_stdout(buf)
saved = sys.stdout
with redir:
    print("Hello", end=" ")
with redir:
    print("World!")
assert sys.stdout is saved, "stdout must be restored after reuse"
assert buf.getvalue() == "Hello World!\n", repr(buf.getvalue())

print("redirect_stdout_reusable_restores_stream OK")
