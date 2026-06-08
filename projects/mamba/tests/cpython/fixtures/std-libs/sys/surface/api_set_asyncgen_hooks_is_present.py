# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_set_asyncgen_hooks_is_present"
# subject = "sys.set_asyncgen_hooks"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.set_asyncgen_hooks: api_set_asyncgen_hooks_is_present (surface)."""
import sys

assert hasattr(sys, "set_asyncgen_hooks")
print("api_set_asyncgen_hooks_is_present OK")
