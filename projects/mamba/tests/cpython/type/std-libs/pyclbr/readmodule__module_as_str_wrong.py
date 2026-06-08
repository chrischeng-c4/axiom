# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyclbr"
# dimension = "type"
# case = "readmodule__module_as_str_wrong"
# subject = "pyclbr.readmodule(module: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pyclbr.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pyclbr.readmodule(module: str); call it with the wrong type.

typeshed contract: module is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pyclbr import readmodule
try:
    readmodule(12345)  # module: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
