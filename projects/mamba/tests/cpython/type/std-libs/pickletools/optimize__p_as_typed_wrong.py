# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "optimize__p_as_typed_wrong"
# subject = "pickletools.optimize(p: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.optimize(p: typed); call it with the wrong type.

typeshed contract: p is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import optimize
try:
    optimize(_W())  # p: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
