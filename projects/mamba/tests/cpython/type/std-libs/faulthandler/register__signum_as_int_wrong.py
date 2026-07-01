# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "faulthandler"
# dimension = "type"
# case = "register__signum_as_int_wrong"
# subject = "faulthandler.register(signum: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/faulthandler.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: faulthandler.register(signum: int); call it with the wrong type.

typeshed contract: signum is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from faulthandler import register
try:
    register("not_an_int")  # signum: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
