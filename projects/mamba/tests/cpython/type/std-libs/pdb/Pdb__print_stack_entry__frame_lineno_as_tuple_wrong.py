# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "type"
# case = "Pdb__print_stack_entry__frame_lineno_as_tuple_wrong"
# subject = "pdb.Pdb.print_stack_entry(frame_lineno: tuple)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed frame_lineno"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pdb.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed frame_lineno
# mamba-strict-type: TypeError
"""Type wall: pdb.Pdb.print_stack_entry(frame_lineno: tuple); call it with the wrong type.

typeshed contract: frame_lineno is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pdb import Pdb
obj = object.__new__(Pdb)
try:
    obj.print_stack_entry(12345)  # frame_lineno: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
