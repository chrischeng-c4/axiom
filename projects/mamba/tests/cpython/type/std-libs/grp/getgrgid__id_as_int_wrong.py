# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "grp"
# dimension = "type"
# case = "getgrgid__id_as_int_wrong"
# subject = "grp.getgrgid(id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/grp.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: grp.getgrgid(id: int); call it with the wrong type.

typeshed contract: id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from grp import getgrgid
try:
    getgrgid("not_an_int")  # id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
