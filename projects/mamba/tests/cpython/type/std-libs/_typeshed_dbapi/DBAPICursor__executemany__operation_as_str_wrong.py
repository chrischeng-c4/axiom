# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_typeshed_dbapi"
# dimension = "type"
# case = "DBAPICursor__executemany__operation_as_str_wrong"
# subject = "_typeshed.dbapi.DBAPICursor.executemany(operation: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_typeshed/dbapi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _typeshed.dbapi.DBAPICursor.executemany(operation: str); call it with the wrong type.

typeshed contract: operation is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _typeshed.dbapi import DBAPICursor
obj = object.__new__(DBAPICursor)
try:
    obj.executemany(12345, None)  # operation: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
