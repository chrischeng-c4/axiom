# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_util"
# dimension = "type"
# case = "LazyLoader__factory__loader_as_Loader_wrong"
# subject = "importlib.util.LazyLoader.factory(loader: Loader)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.util.LazyLoader.factory(loader: Loader); call it with the wrong type.

typeshed contract: loader is Loader. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.util import LazyLoader
try:
    LazyLoader.factory(_W())  # loader: Loader <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
