# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "weakmethod_is_callable"
# subject = "weakref.WeakMethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""weakref.WeakMethod: weakmethod_is_callable (surface)."""
import weakref

assert callable(weakref.WeakMethod)
print("weakmethod_is_callable OK")
