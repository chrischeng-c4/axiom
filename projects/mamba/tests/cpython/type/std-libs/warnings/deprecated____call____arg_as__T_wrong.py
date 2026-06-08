# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "type"
# case = "deprecated____call____arg_as__T_wrong"
# subject = "warnings.deprecated.__call__(arg: _T)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed arg"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/warnings.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed arg
# mamba-strict-type: TypeError
"""Type wall: warnings.deprecated.__call__(arg: _T); call it with the wrong type.

typeshed contract: arg is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from warnings import deprecated
obj = object.__new__(deprecated)
try:
    obj.__call__(_W())  # arg: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
