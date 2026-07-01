# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_managers"
# dimension = "type"
# case = "Token____setstate____state_as_tuple_wrong"
# subject = "multiprocessing.managers.Token.__setstate__(state: tuple)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/managers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.managers.Token.__setstate__(state: tuple); call it with the wrong type.

typeshed contract: state is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.managers import Token
obj = object.__new__(Token)
try:
    obj.__setstate__(12345)  # state: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
