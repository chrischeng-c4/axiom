# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "surface"
# case = "wrap_is_callable"
# subject = "textwrap.wrap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.wrap: wrap_is_callable (surface)."""
import textwrap

assert callable(textwrap.wrap)
print("wrap_is_callable OK")
