# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "surface"
# case = "redirect_stderr_is_callable"
# subject = "contextlib.redirect_stderr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextlib.redirect_stderr: redirect_stderr_is_callable (surface)."""
import contextlib

assert callable(contextlib.redirect_stderr)
print("redirect_stderr_is_callable OK")
