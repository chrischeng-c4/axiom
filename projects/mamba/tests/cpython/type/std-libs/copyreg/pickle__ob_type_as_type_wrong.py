# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copyreg"
# dimension = "type"
# case = "pickle__ob_type_as_type_wrong"
# subject = "copyreg.pickle(ob_type: type)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/copyreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: copyreg.pickle(ob_type: type); call it with the wrong type.

typeshed contract: ob_type is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from copyreg import pickle
try:
    pickle(_W(), None)  # ob_type: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
