# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pwd"
# dimension = "type"
# case = "getpwuid__uid_as_int_wrong"
# subject = "pwd.getpwuid(uid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pwd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pwd.getpwuid(uid: int); call it with the wrong type.

typeshed contract: uid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pwd import getpwuid
try:
    getpwuid("not_an_int")  # uid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
