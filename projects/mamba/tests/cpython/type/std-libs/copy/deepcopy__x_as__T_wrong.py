# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "type"
# case = "deepcopy__x_as__T_wrong"
# subject = "copy.deepcopy(x: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/copy.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: copy.deepcopy(x: _T); call it with the wrong type.

typeshed contract: x is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from copy import deepcopy
try:
    deepcopy(_W())  # x: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
