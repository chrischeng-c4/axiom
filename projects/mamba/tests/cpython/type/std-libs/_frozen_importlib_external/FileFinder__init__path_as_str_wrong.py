# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib_external"
# dimension = "type"
# case = "FileFinder__init__path_as_str_wrong"
# subject = "_frozen_importlib_external.FileFinder.__init__(path: str)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed path"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib_external.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed path
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib_external.FileFinder.__init__(path: str); call it with the wrong type.

typeshed contract: path is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib_external import FileFinder
try:
    FileFinder(12345)  # path: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
