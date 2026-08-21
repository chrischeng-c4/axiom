# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "getweakrefs_is_callable"
# subject = "weakref.getweakrefs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""weakref.getweakrefs: getweakrefs_is_callable (surface)."""
import weakref

assert callable(weakref.getweakrefs)
print("getweakrefs_is_callable OK")
