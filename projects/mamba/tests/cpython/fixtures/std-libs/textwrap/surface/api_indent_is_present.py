# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "surface"
# case = "api_indent_is_present"
# subject = "textwrap.indent"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""textwrap.indent: api_indent_is_present (surface)."""
import textwrap

assert hasattr(textwrap, "indent")
print("api_indent_is_present OK")
