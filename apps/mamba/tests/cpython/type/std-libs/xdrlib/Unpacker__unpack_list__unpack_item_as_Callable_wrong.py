# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Unpacker__unpack_list__unpack_item_as_Callable_wrong"
# subject = "xdrlib.Unpacker.unpack_list(unpack_item: Callable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed unpack_item"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed unpack_item
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Unpacker.unpack_list(unpack_item: Callable); call it with the wrong type.

typeshed contract: unpack_item is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xdrlib import Unpacker
obj = object.__new__(Unpacker)
try:
    obj.unpack_list(_W())  # unpack_item: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
