# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "proxytypes_attr"
# subject = "weakref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""weakref: proxytypes_attr (surface)."""
import weakref

assert hasattr(weakref, "ProxyTypes")
print("proxytypes_attr OK")
