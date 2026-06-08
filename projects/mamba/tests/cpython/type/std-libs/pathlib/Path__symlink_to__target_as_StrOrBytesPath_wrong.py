# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__symlink_to__target_as_StrOrBytesPath_wrong"
# subject = "pathlib.Path.symlink_to(target: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.symlink_to(target: StrOrBytesPath); call it with the wrong type.

typeshed contract: target is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pathlib import Path
obj = object.__new__(Path)
try:
    obj.symlink_to(_W())  # target: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
