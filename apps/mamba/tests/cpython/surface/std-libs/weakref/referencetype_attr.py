# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "referencetype_attr"
# subject = "weakref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""weakref: referencetype_attr (surface)."""
import weakref

assert hasattr(weakref, "ReferenceType")
print("referencetype_attr OK")
