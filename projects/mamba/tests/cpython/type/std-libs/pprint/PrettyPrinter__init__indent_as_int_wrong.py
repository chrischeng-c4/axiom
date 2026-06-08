# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "type"
# case = "PrettyPrinter__init__indent_as_int_wrong"
# subject = "pprint.PrettyPrinter.__init__(indent: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pprint.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pprint.PrettyPrinter.__init__(indent: int); call it with the wrong type.

typeshed contract: indent is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pprint import PrettyPrinter
try:
    PrettyPrinter("not_an_int")  # indent: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
