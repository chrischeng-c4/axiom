# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources__common"
# dimension = "type"
# case = "files__package_as_Package_wrong"
# subject = "importlib.resources._common.files(package: Package)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed package"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/_common.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed package
# mamba-strict-type: TypeError
"""Type wall: importlib.resources._common.files(package: Package); call it with the wrong type.

typeshed contract: package is Package. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources._common import files
try:
    files(_W())  # package: Package <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
