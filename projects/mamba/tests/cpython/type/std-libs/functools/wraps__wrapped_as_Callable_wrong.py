# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "type"
# case = "wraps__wrapped_as_Callable_wrong"
# subject = "functools.wraps(wrapped: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/functools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: functools.wraps(wrapped: Callable); call it with the wrong type.

typeshed contract: wrapped is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from functools import wraps
try:
    wraps(_W())  # wrapped: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
