# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources"
# dimension = "type"
# case = "is_resource__package_as_Package_wrong"
# subject = "importlib.resources.is_resource(package: Package)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources.is_resource(package: Package); call it with the wrong type.

typeshed contract: package is Package. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources import is_resource
try:
    is_resource(_W(), "")  # package: Package <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
