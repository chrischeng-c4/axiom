# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "type"
# case = "Connection__set_progress_handler__progress_handler_as_typed_wrong"
# subject = "sqlite3.Connection.set_progress_handler(progress_handler: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed progress_handler"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sqlite3.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed progress_handler
# mamba-strict-type: TypeError
"""Type wall: sqlite3.Connection.set_progress_handler(progress_handler: typed); call it with the wrong type.

typeshed contract: progress_handler is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sqlite3 import Connection
obj = object.__new__(Connection)
try:
    obj.set_progress_handler(_W(), 0)  # progress_handler: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
