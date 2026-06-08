# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_sqlite3"
# dimension = "type"
# case = "register_adapter__type_as_type_wrong"
# subject = "_sqlite3.register_adapter(type: type)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed type"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_sqlite3.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed type
# mamba-strict-type: TypeError
"""Type wall: _sqlite3.register_adapter(type: type); call it with the wrong type.

typeshed contract: type is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _sqlite3 import register_adapter
try:
    register_adapter(_W(), None)  # type: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
