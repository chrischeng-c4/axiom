# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "sin__z_as__C_wrong"
# subject = "cmath.sin(z: _C)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed z"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed z
# mamba-strict-type: TypeError
"""Type wall: cmath.sin(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import sin
try:
    sin(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
