# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "ref_is_callable"
# subject = "weakref.ref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""weakref.ref: ref_is_callable (surface)."""
import weakref

assert callable(weakref.ref)
print("ref_is_callable OK")
