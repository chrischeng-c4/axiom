# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "type"
# case = "nlargest__n_as_int_wrong"
# subject = "heapq.nlargest(n: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/heapq.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: heapq.nlargest(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from heapq import nlargest
try:
    nlargest("not_an_int", None)  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
