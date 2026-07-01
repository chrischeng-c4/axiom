# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_bisect"
# dimension = "type"
# case = "bisect_right__a_as_SupportsGetItem_wrong"
# subject = "_bisect.bisect_right(a: SupportsGetItem)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_bisect.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _bisect.bisect_right(a: SupportsGetItem); call it with the wrong type.

typeshed contract: a is SupportsGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _bisect import bisect_right
try:
    bisect_right(_W(), None, 0, 0)  # a: SupportsGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
