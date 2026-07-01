# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_sqlite3"
# dimension = "type"
# case = "complete_statement__statement_as_str_wrong"
# subject = "_sqlite3.complete_statement(statement: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_sqlite3.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _sqlite3.complete_statement(statement: str); call it with the wrong type.

typeshed contract: statement is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _sqlite3 import complete_statement
try:
    complete_statement(12345)  # statement: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
