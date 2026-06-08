# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "surface"
# case = "api_fill_is_present"
# subject = "textwrap.fill"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""textwrap.fill: api_fill_is_present (surface)."""
import textwrap

assert hasattr(textwrap, "fill")
print("api_fill_is_present OK")
