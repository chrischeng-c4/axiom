# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "type"
# case = "Path__resolve__strict_as_bool_wrong"
# subject = "pathlib.Path.resolve(strict: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pathlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pathlib.Path.resolve(strict: bool); call it with the wrong type.

typeshed contract: strict is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pathlib import Path
obj = object.__new__(Path)
try:
    obj.resolve("not_a_bool")  # strict: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
