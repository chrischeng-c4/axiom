# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources_abc"
# dimension = "type"
# case = "ResourceReader__open_resource__resource_as_str_wrong"
# subject = "importlib.resources.abc.ResourceReader.open_resource(resource: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources.abc.ResourceReader.open_resource(resource: str); call it with the wrong type.

typeshed contract: resource is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from importlib.resources.abc import ResourceReader
obj = object.__new__(ResourceReader)
try:
    obj.open_resource(12345)  # resource: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
