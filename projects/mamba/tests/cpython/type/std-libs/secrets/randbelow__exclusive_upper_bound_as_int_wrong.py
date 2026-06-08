# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "type"
# case = "randbelow__exclusive_upper_bound_as_int_wrong"
# subject = "secrets.randbelow(exclusive_upper_bound: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/secrets.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: secrets.randbelow(exclusive_upper_bound: int); call it with the wrong type.

typeshed contract: exclusive_upper_bound is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from secrets import randbelow
try:
    randbelow("not_an_int")  # exclusive_upper_bound: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
