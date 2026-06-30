# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakref"
# dimension = "surface"
# case = "proxy_accepts_user_object"
# subject = "_weakref.proxy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""_weakref.proxy accepts weak-referenceable user objects."""
from _weakref import proxy


class Target:
    pass


obj = Target()
p = proxy(obj)
assert p is not None
print("_weakref_proxy_accepts_user_object OK")
