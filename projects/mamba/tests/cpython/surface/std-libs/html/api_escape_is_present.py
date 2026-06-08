# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html"
# dimension = "surface"
# case = "api_escape_is_present"
# subject = "html.escape"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""html.escape: api_escape_is_present (surface)."""
import html

assert hasattr(html, "escape")
print("api_escape_is_present OK")
