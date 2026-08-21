# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources__common"
# dimension = "type"
# case = "get_resource_reader__package_as_ModuleType_wrong"
# subject = "importlib.resources._common.get_resource_reader(package: ModuleType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/_common.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources._common.get_resource_reader(package: ModuleType); call it with the wrong type.

typeshed contract: package is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources._common import get_resource_reader
try:
    get_resource_reader(_W())  # package: ModuleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
