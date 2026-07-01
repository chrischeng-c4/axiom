# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "type"
# case = "OrderedDict__popitem__last_as_bool_wrong"
# subject = "collections.OrderedDict.popitem(last: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/collections.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: collections.OrderedDict.popitem(last: bool); call it with the wrong type.

typeshed contract: last is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from collections import OrderedDict
obj = object.__new__(OrderedDict)
try:
    obj.popitem("not_a_bool")  # last: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
