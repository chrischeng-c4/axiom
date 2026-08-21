# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_imp"
# dimension = "type"
# case = "create_dynamic__spec_as_ModuleSpec_wrong"
# subject = "_imp.create_dynamic(spec: ModuleSpec)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_imp.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _imp.create_dynamic(spec: ModuleSpec); call it with the wrong type.

typeshed contract: spec is ModuleSpec. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _imp import create_dynamic
try:
    create_dynamic(_W())  # spec: ModuleSpec <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
