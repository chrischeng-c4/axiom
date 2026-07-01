# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_msi"
# dimension = "type"
# case = "OpenDatabase__path_as_str_wrong"
# subject = "_msi.OpenDatabase(path: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_msi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _msi.OpenDatabase(path: str); call it with the wrong type.

typeshed contract: path is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _msi import OpenDatabase
try:
    OpenDatabase(12345, 0)  # path: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
