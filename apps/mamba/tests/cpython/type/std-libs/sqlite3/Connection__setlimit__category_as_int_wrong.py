# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "type"
# case = "Connection__setlimit__category_as_int_wrong"
# subject = "sqlite3.Connection.setlimit(category: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sqlite3.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sqlite3.Connection.setlimit(category: int); call it with the wrong type.

typeshed contract: category is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sqlite3 import Connection
obj = object.__new__(Connection)
try:
    obj.setlimit("not_an_int", 0)  # category: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
