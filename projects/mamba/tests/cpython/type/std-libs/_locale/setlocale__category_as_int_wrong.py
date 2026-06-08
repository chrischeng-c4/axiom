# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "type"
# case = "setlocale__category_as_int_wrong"
# subject = "_locale.setlocale(category: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _locale.setlocale(category: int); call it with the wrong type.

typeshed contract: category is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _locale import setlocale
try:
    setlocale("not_an_int")  # category: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
