# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "PurePath__is_relative_to__other_as_StrPath_wrong"
# subject = "pathlib.PurePath.is_relative_to(other: StrPath)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed other"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed other
# mamba-strict-type: TypeError
"""Type wall: pathlib.PurePath.is_relative_to(other: StrPath); call it with the wrong type.

typeshed contract: other is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import PurePath
obj = object.__new__(PurePath)
try:
    obj.is_relative_to(_W())  # other: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
