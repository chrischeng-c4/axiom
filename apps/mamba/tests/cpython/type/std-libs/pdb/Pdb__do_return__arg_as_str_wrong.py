# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "type"
# case = "Pdb__do_return__arg_as_str_wrong"
# subject = "pdb.Pdb.do_return(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pdb.Pdb.do_return(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pdb import Pdb
obj = object.__new__(Pdb)
try:
    obj.do_return(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
