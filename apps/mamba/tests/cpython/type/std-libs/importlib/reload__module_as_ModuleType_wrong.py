# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "type"
# case = "reload__module_as_ModuleType_wrong"
# subject = "importlib.reload(module: ModuleType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.reload(module: ModuleType); call it with the wrong type.

typeshed contract: module is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib import reload
try:
    reload(_W())  # module: ModuleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
