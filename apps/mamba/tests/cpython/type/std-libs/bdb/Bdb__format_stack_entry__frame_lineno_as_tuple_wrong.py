# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__format_stack_entry__frame_lineno_as_tuple_wrong"
# subject = "bdb.Bdb.format_stack_entry(frame_lineno: tuple)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.format_stack_entry(frame_lineno: tuple); call it with the wrong type.

typeshed contract: frame_lineno is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.format_stack_entry(12345)  # frame_lineno: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
