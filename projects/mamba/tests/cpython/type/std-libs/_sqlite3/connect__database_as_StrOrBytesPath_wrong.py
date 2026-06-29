# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_sqlite3"
# dimension = "type"
# case = "connect__database_as_StrOrBytesPath_wrong"
# subject = "_sqlite3.connect(database: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_sqlite3.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _sqlite3.connect(database: StrOrBytesPath); call it with the wrong type.

typeshed contract: database is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _sqlite3 import connect
try:
    connect(_W())  # database: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
