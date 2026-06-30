# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "type"
# case = "total_ordering__cls_as_type_wrong"
# subject = "functools.total_ordering(cls: type)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/functools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: functools.total_ordering(cls: type); call it with the wrong type.

typeshed contract: cls is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from functools import total_ordering
try:
    total_ordering(_W())  # cls: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
