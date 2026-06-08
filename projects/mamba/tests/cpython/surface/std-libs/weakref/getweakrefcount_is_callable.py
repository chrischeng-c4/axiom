# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "getweakrefcount_is_callable"
# subject = "weakref.getweakrefcount"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""weakref.getweakrefcount: getweakrefcount_is_callable (surface)."""
import weakref

assert callable(weakref.getweakrefcount)
print("getweakrefcount_is_callable OK")
