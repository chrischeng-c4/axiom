# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "open_is_callable"
# subject = "io.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.open: open_is_callable (surface)."""
import io

assert callable(io.open)
print("open_is_callable OK")
