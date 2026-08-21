# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "type"
# case = "Connection__execute__sql_as_str_wrong"
# subject = "sqlite3.Connection.execute(sql: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sqlite3.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sqlite3.Connection.execute(sql: str); call it with the wrong type.

typeshed contract: sql is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sqlite3 import Connection
obj = object.__new__(Connection)
try:
    obj.execute(12345)  # sql: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
