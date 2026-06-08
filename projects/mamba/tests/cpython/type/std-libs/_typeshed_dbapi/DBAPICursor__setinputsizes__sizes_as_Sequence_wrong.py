# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_typeshed_dbapi"
# dimension = "type"
# case = "DBAPICursor__setinputsizes__sizes_as_Sequence_wrong"
# subject = "_typeshed.dbapi.DBAPICursor.setinputsizes(sizes: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sizes"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_typeshed/dbapi.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sizes
# mamba-strict-type: TypeError
"""Type wall: _typeshed.dbapi.DBAPICursor.setinputsizes(sizes: Sequence); call it with the wrong type.

typeshed contract: sizes is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _typeshed.dbapi import DBAPICursor
obj = object.__new__(DBAPICursor)
try:
    obj.setinputsizes(_W())  # sizes: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
