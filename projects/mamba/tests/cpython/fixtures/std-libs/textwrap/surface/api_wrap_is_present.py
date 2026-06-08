# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "surface"
# case = "api_wrap_is_present"
# subject = "textwrap.wrap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""textwrap.wrap: api_wrap_is_present (surface)."""
import textwrap

assert hasattr(textwrap, "wrap")
print("api_wrap_is_present OK")
