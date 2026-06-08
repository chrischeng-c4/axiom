# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "Hook____call____etype_as_typed_wrong"
# subject = "cgitb.Hook.__call__(etype: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed etype"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed etype
# mamba-strict-type: TypeError
"""Type wall: cgitb.Hook.__call__(etype: typed); call it with the wrong type.

typeshed contract: etype is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgitb import Hook
obj = object.__new__(Hook)
try:
    obj.__call__(_W(), None, None)  # etype: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
