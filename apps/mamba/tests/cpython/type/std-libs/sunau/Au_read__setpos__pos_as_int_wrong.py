# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sunau"
# dimension = "type"
# case = "Au_read__setpos__pos_as_int_wrong"
# subject = "sunau.Au_read.setpos(pos: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sunau.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sunau.Au_read.setpos(pos: int); call it with the wrong type.

typeshed contract: pos is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sunau import Au_read
obj = object.__new__(Au_read)
try:
    obj.setpos("not_an_int")  # pos: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
