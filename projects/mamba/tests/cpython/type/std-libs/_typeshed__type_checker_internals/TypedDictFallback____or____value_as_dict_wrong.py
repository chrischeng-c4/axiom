# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_typeshed__type_checker_internals"
# dimension = "type"
# case = "TypedDictFallback____or____value_as_dict_wrong"
# subject = "_typeshed._type_checker_internals.TypedDictFallback.__or__(value: dict)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed value"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_typeshed/_type_checker_internals.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed value
# mamba-strict-type: TypeError
"""Type wall: _typeshed._type_checker_internals.TypedDictFallback.__or__(value: dict); call it with the wrong type.

typeshed contract: value is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _typeshed._type_checker_internals import TypedDictFallback
obj = object.__new__(TypedDictFallback)
try:
    obj.__or__(12345)  # value: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
