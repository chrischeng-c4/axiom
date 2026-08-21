# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Packer__pack_list__list_as_Sequence_wrong"
# subject = "xdrlib.Packer.pack_list(list: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed list"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed list
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Packer.pack_list(list: Sequence); call it with the wrong type.

typeshed contract: list is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xdrlib import Packer
obj = object.__new__(Packer)
try:
    obj.pack_list(_W(), None)  # list: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
