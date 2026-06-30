# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "type"
# case = "unregister__func_as_Callable_wrong"
# subject = "atexit.unregister(func: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/atexit.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: atexit.unregister(func: Callable); call it with the wrong type.

typeshed contract: func is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from atexit import unregister
try:
    unregister(_W())  # func: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
