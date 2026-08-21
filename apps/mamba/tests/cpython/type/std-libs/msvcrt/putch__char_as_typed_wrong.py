# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msvcrt"
# dimension = "type"
# case = "putch__char_as_typed_wrong"
# subject = "msvcrt.putch(char: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msvcrt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msvcrt.putch(char: typed); call it with the wrong type.

typeshed contract: char is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from msvcrt import putch
try:
    putch(_W())  # char: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
