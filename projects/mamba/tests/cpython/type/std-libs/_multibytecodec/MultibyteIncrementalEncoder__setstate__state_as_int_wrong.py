# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteIncrementalEncoder__setstate__state_as_int_wrong"
# subject = "_multibytecodec.MultibyteIncrementalEncoder.setstate(state: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteIncrementalEncoder.setstate(state: int); call it with the wrong type.

typeshed contract: state is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _multibytecodec import MultibyteIncrementalEncoder
obj = object.__new__(MultibyteIncrementalEncoder)
try:
    obj.setstate("not_an_int")  # state: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
