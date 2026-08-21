# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_abc"
# dimension = "type"
# case = "FileLoader__init__fullname_as_str_wrong"
# subject = "importlib.abc.FileLoader.__init__(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.abc.FileLoader.__init__(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.abc import FileLoader
try:
    FileLoader(12345, "")  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
