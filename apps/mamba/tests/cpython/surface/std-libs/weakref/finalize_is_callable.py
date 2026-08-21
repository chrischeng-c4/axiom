# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "finalize_is_callable"
# subject = "weakref.finalize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""weakref.finalize: finalize_is_callable (surface)."""
import weakref

assert callable(weakref.finalize)
print("finalize_is_callable OK")
