# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msilib"
# dimension = "type"
# case = "add_tables__db_as__Database_wrong"
# subject = "msilib.add_tables(db: _Database)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msilib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msilib.add_tables(db: _Database); call it with the wrong type.

typeshed contract: db is _Database. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from msilib import add_tables
try:
    add_tables(_W(), None)  # db: _Database <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
