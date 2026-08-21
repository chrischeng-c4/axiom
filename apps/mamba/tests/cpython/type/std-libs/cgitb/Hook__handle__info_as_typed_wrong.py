# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "Hook__handle__info_as_typed_wrong"
# subject = "cgitb.Hook.handle(info: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgitb.Hook.handle(info: typed); call it with the wrong type.

typeshed contract: info is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgitb import Hook
obj = object.__new__(Hook)
try:
    obj.handle(_W())  # info: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
