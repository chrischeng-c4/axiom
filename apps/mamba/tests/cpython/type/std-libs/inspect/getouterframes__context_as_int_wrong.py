# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "type"
# case = "getouterframes__context_as_int_wrong"
# subject = "inspect.getouterframes(context: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/inspect.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: inspect.getouterframes(context: int); call it with the wrong type.

typeshed contract: context is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from inspect import getouterframes
try:
    getouterframes(None, "not_an_int")  # context: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
