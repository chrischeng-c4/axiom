# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "type"
# case = "ParsingError__append__lineno_as_int_wrong"
# subject = "configparser.ParsingError.append(lineno: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/configparser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: configparser.ParsingError.append(lineno: int); call it with the wrong type.

typeshed contract: lineno is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from configparser import ParsingError
obj = object.__new__(ParsingError)
try:
    obj.append("not_an_int", "")  # lineno: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
