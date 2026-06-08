# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "type"
# case = "Cursor__setoutputsize__size_as_Unused_wrong"
# subject = "sqlite3.Cursor.setoutputsize(size: Unused)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed size"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sqlite3.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed size
# mamba-strict-type: TypeError
"""Type wall: sqlite3.Cursor.setoutputsize(size: Unused); call it with the wrong type.

typeshed contract: size is Unused. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sqlite3 import Cursor
obj = object.__new__(Cursor)
try:
    obj.setoutputsize(_W())  # size: Unused <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
