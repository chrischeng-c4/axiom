# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources_simple"
# dimension = "type"
# case = "ResourceHandle__open__mode_as_str_wrong"
# subject = "importlib.resources.simple.ResourceHandle.open(mode: str)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed mode"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/simple.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed mode
# mamba-strict-type: TypeError
"""Type wall: importlib.resources.simple.ResourceHandle.open(mode: str); call it with the wrong type.

typeshed contract: mode is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.resources.simple import ResourceHandle
obj = object.__new__(ResourceHandle)
try:
    obj.open(12345)  # mode: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
