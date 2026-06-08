# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "type"
# case = "Connection__enable_load_extension__enable_as_bool_wrong"
# subject = "sqlite3.Connection.enable_load_extension(enable: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed enable"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sqlite3.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed enable
# mamba-strict-type: TypeError
"""Type wall: sqlite3.Connection.enable_load_extension(enable: bool); call it with the wrong type.

typeshed contract: enable is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sqlite3 import Connection
obj = object.__new__(Connection)
try:
    obj.enable_load_extension("not_a_bool")  # enable: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
