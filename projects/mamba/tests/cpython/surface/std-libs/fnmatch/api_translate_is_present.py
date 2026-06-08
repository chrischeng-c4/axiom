# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "surface"
# case = "api_translate_is_present"
# subject = "fnmatch.translate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""fnmatch.translate: api_translate_is_present (surface)."""
import fnmatch

assert hasattr(fnmatch, "translate")
print("api_translate_is_present OK")
