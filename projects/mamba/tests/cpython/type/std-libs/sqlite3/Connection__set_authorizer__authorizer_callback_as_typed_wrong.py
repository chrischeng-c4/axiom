# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "type"
# case = "Connection__set_authorizer__authorizer_callback_as_typed_wrong"
# subject = "sqlite3.Connection.set_authorizer(authorizer_callback: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed authorizer_callback"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sqlite3.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed authorizer_callback
# mamba-strict-type: TypeError
"""Type wall: sqlite3.Connection.set_authorizer(authorizer_callback: typed); call it with the wrong type.

typeshed contract: authorizer_callback is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sqlite3 import Connection
obj = object.__new__(Connection)
try:
    obj.set_authorizer(_W())  # authorizer_callback: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
