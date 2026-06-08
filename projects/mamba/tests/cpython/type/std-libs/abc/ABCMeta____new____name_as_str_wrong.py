# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "type"
# case = "ABCMeta____new____name_as_str_wrong"
# subject = "abc.ABCMeta.__new__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: abc.ABCMeta.__new__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from abc import ABCMeta
obj = object.__new__(ABCMeta)
try:
    obj.__new__(12345, None, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
