# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_csv"
# dimension = "type"
# case = "field_size_limit__new_limit_as_int_wrong"
# subject = "_csv.field_size_limit(new_limit: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _csv.field_size_limit(new_limit: int); call it with the wrong type.

typeshed contract: new_limit is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _csv import field_size_limit
try:
    field_size_limit("not_an_int")  # new_limit: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
