# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_decimal"
# dimension = "type"
# case = "IEEEContext__bits_as_int_wrong"
# subject = "_decimal.IEEEContext(bits: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _decimal.IEEEContext(bits: int); call it with the wrong type.

typeshed contract: bits is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _decimal import IEEEContext
try:
    IEEEContext("not_an_int")  # bits: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
