# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cProfile"
# dimension = "type"
# case = "Profile__runcall__func_as_Callable_wrong"
# subject = "cProfile.Profile.runcall(func: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cProfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cProfile.Profile.runcall(func: Callable); call it with the wrong type.

typeshed contract: func is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cProfile import Profile
obj = object.__new__(Profile)
try:
    obj.runcall(_W())  # func: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
