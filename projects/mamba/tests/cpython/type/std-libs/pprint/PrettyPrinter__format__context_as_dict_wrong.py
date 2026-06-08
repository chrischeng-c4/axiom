# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "type"
# case = "PrettyPrinter__format__context_as_dict_wrong"
# subject = "pprint.PrettyPrinter.format(context: dict)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed context"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pprint.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed context
# mamba-strict-type: TypeError
"""Type wall: pprint.PrettyPrinter.format(context: dict); call it with the wrong type.

typeshed contract: context is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pprint import PrettyPrinter
obj = object.__new__(PrettyPrinter)
try:
    obj.format(None, 12345, 0, 0)  # context: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
