# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html"
# dimension = "surface"
# case = "api_unescape_is_present"
# subject = "html.unescape"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""html.unescape: api_unescape_is_present (surface)."""
import html

assert hasattr(html, "unescape")
print("api_unescape_is_present OK")
