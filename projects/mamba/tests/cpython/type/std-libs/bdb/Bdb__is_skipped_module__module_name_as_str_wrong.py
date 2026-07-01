# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__is_skipped_module__module_name_as_str_wrong"
# subject = "bdb.Bdb.is_skipped_module(module_name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.is_skipped_module(module_name: str); call it with the wrong type.

typeshed contract: module_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.is_skipped_module(12345)  # module_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
