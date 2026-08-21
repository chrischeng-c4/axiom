# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "type"
# case = "randbits__k_as_int_wrong"
# subject = "secrets.randbits(k: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/secrets.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: secrets.randbits(k: int); call it with the wrong type.

typeshed contract: k is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from secrets import randbits
try:
    randbits("not_an_int")  # k: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
