# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "type"
# case = "update_wrapper__wrapper_as_Callable_wrong"
# subject = "functools.update_wrapper(wrapper: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/functools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: functools.update_wrapper(wrapper: Callable); call it with the wrong type.

typeshed contract: wrapper is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from functools import update_wrapper
try:
    update_wrapper(_W(), None)  # wrapper: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
