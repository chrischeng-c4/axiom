# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "type"
# case = "ProxyType____getattr____attr_as_str_wrong"
# subject = "weakref.ProxyType.__getattr__(attr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/weakref.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: weakref.ProxyType.__getattr__(attr: str); call it with the wrong type.

typeshed contract: attr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from weakref import ProxyType
obj = object.__new__(ProxyType)
try:
    obj.__getattr__(12345)  # attr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
