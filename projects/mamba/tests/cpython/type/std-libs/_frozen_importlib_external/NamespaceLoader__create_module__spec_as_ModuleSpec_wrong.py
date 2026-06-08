# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib_external"
# dimension = "type"
# case = "NamespaceLoader__create_module__spec_as_ModuleSpec_wrong"
# subject = "_frozen_importlib_external.NamespaceLoader.create_module(spec: ModuleSpec)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib_external.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib_external.NamespaceLoader.create_module(spec: ModuleSpec); call it with the wrong type.

typeshed contract: spec is ModuleSpec. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _frozen_importlib_external import NamespaceLoader
obj = object.__new__(NamespaceLoader)
try:
    obj.create_module(_W())  # spec: ModuleSpec <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
