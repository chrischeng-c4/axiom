# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path"
# dimension = "type"
# case = "Path__match__path_pattern_as_str_wrong"
# subject = "zipfile._path.Path.match(path_pattern: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.Path.match(path_pattern: str); call it with the wrong type.

typeshed contract: path_pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path import Path
obj = object.__new__(Path)
try:
    obj.match(12345)  # path_pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
