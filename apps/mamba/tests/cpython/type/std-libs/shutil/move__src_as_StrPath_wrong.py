# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "type"
# case = "move__src_as_StrPath_wrong"
# subject = "shutil.move(src: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/shutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: shutil.move(src: StrPath); call it with the wrong type.

typeshed contract: src is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from shutil import move
try:
    move(_W(), None)  # src: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
