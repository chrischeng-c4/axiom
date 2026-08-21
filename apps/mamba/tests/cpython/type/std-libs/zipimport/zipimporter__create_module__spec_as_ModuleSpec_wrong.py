# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__create_module__spec_as_ModuleSpec_wrong"
# subject = "zipimport.zipimporter.create_module(spec: ModuleSpec)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.create_module(spec: ModuleSpec); call it with the wrong type.

typeshed contract: spec is ModuleSpec. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipimport import zipimporter
obj = object.__new__(zipimporter)
try:
    obj.create_module(_W())  # spec: ModuleSpec <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
