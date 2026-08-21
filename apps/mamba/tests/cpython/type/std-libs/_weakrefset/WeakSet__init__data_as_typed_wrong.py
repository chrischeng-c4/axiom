# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet__init__data_as_typed_wrong"
# subject = "_weakrefset.WeakSet.__init__(data: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed data"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed data
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.__init__(data: typed); call it with the wrong type.

typeshed contract: data is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
try:
    WeakSet(_W())  # data: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
