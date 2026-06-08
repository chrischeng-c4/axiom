# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "type"
# case = "Real____round____ndigits_as_int_wrong"
# subject = "numbers.Real.__round__(ndigits: int)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed ndigits"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/numbers.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed ndigits
# mamba-strict-type: TypeError
"""Type wall: numbers.Real.__round__(ndigits: int); call it with the wrong type.

typeshed contract: ndigits is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from numbers import Real
obj = object.__new__(Real)
try:
    obj.__round__("not_an_int")  # ndigits: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
