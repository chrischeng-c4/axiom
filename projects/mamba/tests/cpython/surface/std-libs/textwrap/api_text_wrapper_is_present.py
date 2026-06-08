# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "surface"
# case = "api_text_wrapper_is_present"
# subject = "textwrap.TextWrapper"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""textwrap.TextWrapper: api_text_wrapper_is_present (surface)."""
import textwrap

assert hasattr(textwrap, "TextWrapper")
print("api_text_wrapper_is_present OK")
