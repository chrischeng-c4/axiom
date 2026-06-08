# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet__remove__item_as__T_wrong"
# subject = "_weakrefset.WeakSet.remove(item: _T)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed item"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed item
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.remove(item: _T); call it with the wrong type.

typeshed contract: item is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.remove(_W())  # item: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
