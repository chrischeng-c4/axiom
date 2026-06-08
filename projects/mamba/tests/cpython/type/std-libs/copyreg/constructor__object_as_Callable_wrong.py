# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copyreg"
# dimension = "type"
# case = "constructor__object_as_Callable_wrong"
# subject = "copyreg.constructor(object: Callable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed object"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/copyreg.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed object
# mamba-strict-type: TypeError
"""Type wall: copyreg.constructor(object: Callable); call it with the wrong type.

typeshed contract: object is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from copyreg import constructor
try:
    constructor(_W())  # object: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
