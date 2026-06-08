# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "proxytype_attr"
# subject = "weakref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""weakref: proxytype_attr (surface)."""
import weakref

assert hasattr(weakref, "ProxyType")
print("proxytype_attr OK")
