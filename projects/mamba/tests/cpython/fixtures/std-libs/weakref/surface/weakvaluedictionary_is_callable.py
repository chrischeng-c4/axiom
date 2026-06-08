# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "weakvaluedictionary_is_callable"
# subject = "weakref.WeakValueDictionary"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""weakref.WeakValueDictionary: weakvaluedictionary_is_callable (surface)."""
import weakref

assert callable(weakref.WeakValueDictionary)
print("weakvaluedictionary_is_callable OK")
