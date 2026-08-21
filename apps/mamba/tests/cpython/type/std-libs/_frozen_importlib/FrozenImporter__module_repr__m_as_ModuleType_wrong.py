# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "FrozenImporter__module_repr__m_as_ModuleType_wrong"
# subject = "_frozen_importlib.FrozenImporter.module_repr(m: ModuleType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.FrozenImporter.module_repr(m: ModuleType); call it with the wrong type.

typeshed contract: m is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _frozen_importlib import FrozenImporter
try:
    FrozenImporter.module_repr(_W())  # m: ModuleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
