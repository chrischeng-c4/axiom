# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "resetlocale__category_as_int_wrong"
# subject = "locale.resetlocale(category: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.resetlocale(category: int); call it with the wrong type.

typeshed contract: category is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from locale import resetlocale
try:
    resetlocale("not_an_int")  # category: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
