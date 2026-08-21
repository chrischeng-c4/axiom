# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3_dbapi2"
# dimension = "type"
# case = "DateFromTicks__ticks_as_float_wrong"
# subject = "sqlite3.dbapi2.DateFromTicks(ticks: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sqlite3/dbapi2.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sqlite3.dbapi2.DateFromTicks(ticks: float); call it with the wrong type.

typeshed contract: ticks is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sqlite3.dbapi2 import DateFromTicks
try:
    DateFromTicks("not_a_float")  # ticks: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
