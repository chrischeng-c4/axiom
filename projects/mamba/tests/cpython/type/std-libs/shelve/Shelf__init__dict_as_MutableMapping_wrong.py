# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shelve"
# dimension = "type"
# case = "Shelf__init__dict_as_MutableMapping_wrong"
# subject = "shelve.Shelf.__init__(dict: MutableMapping)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed dict"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/shelve.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed dict
# mamba-strict-type: TypeError
"""Type wall: shelve.Shelf.__init__(dict: MutableMapping); call it with the wrong type.

typeshed contract: dict is MutableMapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from shelve import Shelf
try:
    Shelf(_W())  # dict: MutableMapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
