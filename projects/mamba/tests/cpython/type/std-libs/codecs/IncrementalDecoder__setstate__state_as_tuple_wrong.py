# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "type"
# case = "IncrementalDecoder__setstate__state_as_tuple_wrong"
# subject = "codecs.IncrementalDecoder.setstate(state: tuple)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/codecs.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: codecs.IncrementalDecoder.setstate(state: tuple); call it with the wrong type.

typeshed contract: state is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from codecs import IncrementalDecoder
obj = object.__new__(IncrementalDecoder)
try:
    obj.setstate(12345)  # state: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
