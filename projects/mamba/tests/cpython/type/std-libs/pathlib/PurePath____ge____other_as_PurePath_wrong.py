# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath____ge____other_as_PurePath_wrong"
# subject = "pathlib.PurePath.__ge__(other: PurePath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.__ge__(other: PurePath); call it with the wrong type.

typeshed contract: other is PurePath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import PurePath
obj = object.__new__(PurePath)
try:
    obj.__ge__(_W())  # other: PurePath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
