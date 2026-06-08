# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msvcrt"
# dimension = "type"
# case = "ungetwch__unicode_char_as_str_wrong"
# subject = "msvcrt.ungetwch(unicode_char: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msvcrt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msvcrt.ungetwch(unicode_char: str); call it with the wrong type.

typeshed contract: unicode_char is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from msvcrt import ungetwch
try:
    ungetwch(12345)  # unicode_char: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
