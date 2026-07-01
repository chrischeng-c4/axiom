# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_dbm"
# dimension = "type"
# case = "open__filename_as_str_wrong"
# subject = "_dbm.open(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_dbm.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _dbm.open(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _dbm import open
try:
    open(12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
