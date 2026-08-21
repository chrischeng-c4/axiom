# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_write__setparams__params_as_tuple_wrong"
# subject = "aifc.Aifc_write.setparams(params: tuple)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed params"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed params
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_write.setparams(params: tuple); call it with the wrong type.

typeshed contract: params is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from aifc import Aifc_write
obj = object.__new__(Aifc_write)
try:
    obj.setparams(12345)  # params: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
