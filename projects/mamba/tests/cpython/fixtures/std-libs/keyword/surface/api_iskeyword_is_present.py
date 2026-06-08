# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "surface"
# case = "api_iskeyword_is_present"
# subject = "keyword.iskeyword"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""keyword.iskeyword: api_iskeyword_is_present (surface)."""
import keyword

assert hasattr(keyword, "iskeyword")
print("api_iskeyword_is_present OK")
