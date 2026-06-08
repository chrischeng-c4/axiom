# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "type"
# case = "get_importer__path_item_as_StrOrBytesPath_wrong"
# subject = "pkgutil.get_importer(path_item: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pkgutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pkgutil.get_importer(path_item: StrOrBytesPath); call it with the wrong type.

typeshed contract: path_item is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pkgutil import get_importer
try:
    get_importer(_W())  # path_item: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
