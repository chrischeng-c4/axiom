# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "surface"
# case = "api_shorten_is_present"
# subject = "textwrap.shorten"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""textwrap.shorten: api_shorten_is_present (surface)."""
import textwrap

assert hasattr(textwrap, "shorten")
print("api_shorten_is_present OK")
