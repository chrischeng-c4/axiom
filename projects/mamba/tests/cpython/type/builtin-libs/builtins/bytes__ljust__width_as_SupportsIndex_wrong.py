# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__ljust__width_as_SupportsIndex_wrong"
# subject = "builtins.bytes.ljust(width: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.ljust(width: SupportsIndex); call it with the wrong type.

typeshed contract: width is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.ljust(_W())  # width: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
