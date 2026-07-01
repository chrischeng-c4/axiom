# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "type"
# case = "getrusage__who_as_int_wrong"
# subject = "resource.getrusage(who: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/resource.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: resource.getrusage(who: int); call it with the wrong type.

typeshed contract: who is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from resource import getrusage
try:
    getrusage("not_an_int")  # who: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
