# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "api_getweakrefs_is_present"
# subject = "weakref.getweakrefs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""weakref.getweakrefs: api_getweakrefs_is_present (surface)."""
import weakref

assert hasattr(weakref, "getweakrefs")
print("api_getweakrefs_is_present OK")
