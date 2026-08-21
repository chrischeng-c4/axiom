# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "surface"
# case = "indent_is_callable"
# subject = "textwrap.indent"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.indent: indent_is_callable (surface)."""
import textwrap

assert callable(textwrap.indent)
print("indent_is_callable OK")
