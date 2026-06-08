# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "surface"
# case = "redirect_stdout_is_callable"
# subject = "contextlib.redirect_stdout"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextlib.redirect_stdout: redirect_stdout_is_callable (surface)."""
import contextlib

assert callable(contextlib.redirect_stdout)
print("redirect_stdout_is_callable OK")
