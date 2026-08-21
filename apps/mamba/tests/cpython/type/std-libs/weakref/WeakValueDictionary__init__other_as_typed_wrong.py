# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "type"
# case = "WeakValueDictionary__init__other_as_typed_wrong"
# subject = "weakref.WeakValueDictionary.__init__(other: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed other"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/weakref.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed other
# mamba-strict-type: TypeError
"""Type wall: weakref.WeakValueDictionary.__init__(other: typed); call it with the wrong type.

typeshed contract: other is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from weakref import WeakValueDictionary
try:
    WeakValueDictionary(_W())  # other: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
