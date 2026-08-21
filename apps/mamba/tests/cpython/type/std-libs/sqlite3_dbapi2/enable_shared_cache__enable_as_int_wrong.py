# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3_dbapi2"
# dimension = "type"
# case = "enable_shared_cache__enable_as_int_wrong"
# subject = "sqlite3.dbapi2.enable_shared_cache(enable: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sqlite3/dbapi2.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sqlite3.dbapi2.enable_shared_cache(enable: int); call it with the wrong type.

typeshed contract: enable is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sqlite3.dbapi2 import enable_shared_cache
try:
    enable_shared_cache("not_an_int")  # enable: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
