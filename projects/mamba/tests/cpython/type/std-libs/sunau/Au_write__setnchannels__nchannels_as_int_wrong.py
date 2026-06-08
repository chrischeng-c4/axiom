# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sunau"
# dimension = "type"
# case = "Au_write__setnchannels__nchannels_as_int_wrong"
# subject = "sunau.Au_write.setnchannels(nchannels: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sunau.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sunau.Au_write.setnchannels(nchannels: int); call it with the wrong type.

typeshed contract: nchannels is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sunau import Au_write
obj = object.__new__(Au_write)
try:
    obj.setnchannels("not_an_int")  # nchannels: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
