# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "gettempprefix_is_callable"
# subject = "tempfile.gettempprefix"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile.gettempprefix: gettempprefix_is_callable (surface)."""
import tempfile

assert callable(tempfile.gettempprefix)
print("gettempprefix_is_callable OK")
