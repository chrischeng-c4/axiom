# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "callableproxytype_attr"
# subject = "weakref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""weakref: callableproxytype_attr (surface)."""
import weakref

assert hasattr(weakref, "CallableProxyType")
print("callableproxytype_attr OK")
