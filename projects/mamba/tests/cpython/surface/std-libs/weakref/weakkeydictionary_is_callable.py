# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "weakkeydictionary_is_callable"
# subject = "weakref.WeakKeyDictionary"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""weakref.WeakKeyDictionary: weakkeydictionary_is_callable (surface)."""
import weakref

assert callable(weakref.WeakKeyDictionary)
print("weakkeydictionary_is_callable OK")
