# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "weakset_is_callable"
# subject = "weakref.WeakSet"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""weakref.WeakSet: weakset_is_callable (surface)."""
import weakref

assert callable(weakref.WeakSet)
print("weakset_is_callable OK")
