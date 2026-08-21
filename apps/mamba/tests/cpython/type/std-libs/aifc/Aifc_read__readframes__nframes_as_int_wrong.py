# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_read__readframes__nframes_as_int_wrong"
# subject = "aifc.Aifc_read.readframes(nframes: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_read.readframes(nframes: int); call it with the wrong type.

typeshed contract: nframes is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from aifc import Aifc_read
obj = object.__new__(Aifc_read)
try:
    obj.readframes("not_an_int")  # nframes: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
