# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "surface"
# case = "fill_is_callable"
# subject = "textwrap.fill"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.fill: fill_is_callable (surface)."""
import textwrap

assert callable(textwrap.fill)
print("fill_is_callable OK")
