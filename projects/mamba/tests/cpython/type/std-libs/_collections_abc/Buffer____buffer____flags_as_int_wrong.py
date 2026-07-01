# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_collections_abc"
# dimension = "type"
# case = "Buffer____buffer____flags_as_int_wrong"
# subject = "_collections_abc.Buffer.__buffer__(flags: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_collections_abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _collections_abc.Buffer.__buffer__(flags: int); call it with the wrong type.

typeshed contract: flags is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _collections_abc import Buffer
obj = object.__new__(Buffer)
try:
    obj.__buffer__("not_an_int")  # flags: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
