# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "type"
# case = "WeakValueDictionary____getitem____key_as__KT_wrong"
# subject = "weakref.WeakValueDictionary.__getitem__(key: _KT)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed key"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/weakref.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed key
# mamba-strict-type: TypeError
"""Type wall: weakref.WeakValueDictionary.__getitem__(key: _KT); call it with the wrong type.

typeshed contract: key is _KT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from weakref import WeakValueDictionary
obj = object.__new__(WeakValueDictionary)
try:
    obj.__getitem__(_W())  # key: _KT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
