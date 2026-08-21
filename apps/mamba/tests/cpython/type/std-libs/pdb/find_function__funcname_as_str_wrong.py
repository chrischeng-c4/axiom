# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "type"
# case = "find_function__funcname_as_str_wrong"
# subject = "pdb.find_function(funcname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pdb.find_function(funcname: str); call it with the wrong type.

typeshed contract: funcname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pdb import find_function
try:
    find_function(12345, "")  # funcname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
