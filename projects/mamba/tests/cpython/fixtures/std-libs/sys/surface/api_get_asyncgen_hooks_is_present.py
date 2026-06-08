# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_get_asyncgen_hooks_is_present"
# subject = "sys.get_asyncgen_hooks"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.get_asyncgen_hooks: api_get_asyncgen_hooks_is_present (surface)."""
import sys

assert hasattr(sys, "get_asyncgen_hooks")
print("api_get_asyncgen_hooks_is_present OK")
