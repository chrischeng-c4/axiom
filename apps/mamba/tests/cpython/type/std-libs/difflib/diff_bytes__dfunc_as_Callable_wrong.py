# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "type"
# case = "diff_bytes__dfunc_as_Callable_wrong"
# subject = "difflib.diff_bytes(dfunc: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/difflib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: difflib.diff_bytes(dfunc: Callable); call it with the wrong type.

typeshed contract: dfunc is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from difflib import diff_bytes
try:
    diff_bytes(_W(), None, None)  # dfunc: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
