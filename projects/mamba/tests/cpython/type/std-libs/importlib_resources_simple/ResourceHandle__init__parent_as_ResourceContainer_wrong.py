# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources_simple"
# dimension = "type"
# case = "ResourceHandle__init__parent_as_ResourceContainer_wrong"
# subject = "importlib.resources.simple.ResourceHandle.__init__(parent: ResourceContainer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/simple.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources.simple.ResourceHandle.__init__(parent: ResourceContainer); call it with the wrong type.

typeshed contract: parent is ResourceContainer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources.simple import ResourceHandle
try:
    ResourceHandle(_W(), "")  # parent: ResourceContainer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
