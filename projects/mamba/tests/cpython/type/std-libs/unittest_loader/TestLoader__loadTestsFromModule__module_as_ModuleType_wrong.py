# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_loader"
# dimension = "type"
# case = "TestLoader__loadTestsFromModule__module_as_ModuleType_wrong"
# subject = "unittest.loader.TestLoader.loadTestsFromModule(module: ModuleType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/loader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.loader.TestLoader.loadTestsFromModule(module: ModuleType); call it with the wrong type.

typeshed contract: module is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.loader import TestLoader
obj = object.__new__(TestLoader)
try:
    obj.loadTestsFromModule(_W())  # module: ModuleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
