# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteIncrementalDecoder__setstate__state_as_tuple_wrong"
# subject = "_multibytecodec.MultibyteIncrementalDecoder.setstate(state: tuple)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed state"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed state
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteIncrementalDecoder.setstate(state: tuple); call it with the wrong type.

typeshed contract: state is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _multibytecodec import MultibyteIncrementalDecoder
obj = object.__new__(MultibyteIncrementalDecoder)
try:
    obj.setstate(12345)  # state: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
