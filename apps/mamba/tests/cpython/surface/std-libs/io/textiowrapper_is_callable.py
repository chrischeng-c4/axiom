# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "textiowrapper_is_callable"
# subject = "io.TextIOWrapper"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.TextIOWrapper: textiowrapper_is_callable (surface)."""
import io

assert callable(io.TextIOWrapper)
print("textiowrapper_is_callable OK")
